//! Embedded HTTP server that serves mock rules.

use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Body as HyperBody, Frame, Incoming, SizeHint};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};

use super::matcher::{find_match, Match};
use super::{
    load_recording_config, load_resources, load_rules, render_body, MockBody, MockError, MockFault,
    MockFaultKind, MockResource, MockRule, RecordingConfig, ScenarioActivation, ScrubConfig,
};
use crate::{http::Method, script::ScriptRuntime};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type MockResponseBody = BoxBody<Bytes, BoxError>;

/// Wire-format payload for the server's request log channel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MockLog {
    pub method: String,
    pub path: String,
    pub matched_rule: Option<String>,
    pub status: u16,
}

/// Rule reload event emitted by the hot-reload server.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MockReload {
    pub rules: usize,
    pub error: Option<String>,
}

#[derive(Clone)]
struct ServerState {
    rules: Arc<RwLock<Vec<MockRule>>>,
    resources: Arc<RwLock<ResourceState>>,
    recording: Option<RecordingState>,
    logs: Arc<RwLock<Vec<MockLog>>>,
}

#[derive(Clone)]
struct RecordingState {
    workspace_root: PathBuf,
    config: Arc<RwLock<RecordingConfig>>,
}

#[derive(Debug, Clone, Default)]
struct ResourceState {
    definitions: Vec<MockResource>,
    seeds: HashMap<String, Vec<serde_json::Value>>,
    data: HashMap<String, Vec<serde_json::Value>>,
}

/// Run the mock server in the foreground, until ctrl-c (or `shutdown_rx` fires).
/// On bind error, returns immediately.
pub async fn run(
    rules: Vec<MockRule>,
    addr: SocketAddr,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    on_log: Option<Arc<dyn Fn(MockLog) + Send + Sync>>,
) -> Result<(), MockError> {
    run_dynamic(
        ServerState::new(rules, ResourceState::default()),
        addr,
        shutdown_rx,
        on_log,
        None,
    )
    .await
}

/// Run the mock server and reload `collections/*/mocks/*.yaml` from disk when
/// their parsed rule set changes. Invalid edits keep the last good rules active.
pub async fn run_with_hot_reload(
    workspace_root: PathBuf,
    addr: SocketAddr,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    on_log: Option<Arc<dyn Fn(MockLog) + Send + Sync>>,
    on_reload: Option<Arc<dyn Fn(MockReload) + Send + Sync>>,
) -> Result<(), MockError> {
    let state = ServerState::new(
        load_rules(&workspace_root)?,
        ResourceState::from_loaded(load_resources(&workspace_root)?),
    )
    .with_recording(
        workspace_root.clone(),
        load_recording_config(&workspace_root)?,
    );
    run_dynamic(
        state,
        addr,
        shutdown_rx,
        on_log,
        Some((workspace_root, on_reload)),
    )
    .await
}

async fn run_dynamic(
    state: ServerState,
    addr: SocketAddr,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    on_log: Option<Arc<dyn Fn(MockLog) + Send + Sync>>,
    reload: Option<(PathBuf, Option<Arc<dyn Fn(MockReload) + Send + Sync>>)>,
) -> Result<(), MockError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| MockError::Server(format!("bind {addr}: {e}")))?;

    let log = on_log.unwrap_or_else(|| Arc::new(|_| {}));
    let mut reload_tick = tokio::time::interval(Duration::from_millis(500));

    let mut shutdown = shutdown_rx;
    loop {
        let accept = listener.accept();
        let conn = tokio::select! {
            res = accept => res,
            _ = reload_tick.tick(), if reload.is_some() => {
                if let Some((workspace_root, on_reload)) = &reload {
                    reload_workspace(
                        workspace_root,
                        &state.rules,
                        &state.resources,
                        state.recording.as_ref(),
                        on_reload,
                    )
                    .await;
                }
                continue;
            }
            _ = wait_shutdown(&mut shutdown) => return Ok(()),
        };

        let (stream, _peer) = match conn {
            Ok(v) => v,
            Err(e) => {
                eprintln!("stubhouse mock: accept error: {e}");
                continue;
            }
        };

        let state = state.clone();
        let log = Arc::clone(&log);
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let state = state.clone();
                let log = Arc::clone(&log);
                async move { handle(state, req, log).await }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("stubhouse mock: connection error: {e}");
            }
        });
    }
}

impl ServerState {
    fn new(rules: Vec<MockRule>, resources: ResourceState) -> Self {
        Self {
            rules: Arc::new(RwLock::new(rules)),
            resources: Arc::new(RwLock::new(resources)),
            recording: None,
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn with_recording(mut self, workspace_root: PathBuf, config: RecordingConfig) -> Self {
        self.recording = Some(RecordingState {
            workspace_root,
            config: Arc::new(RwLock::new(config)),
        });
        self
    }
}

impl ResourceState {
    fn from_loaded(resources: Vec<(MockResource, Vec<serde_json::Value>)>) -> Self {
        let mut definitions = Vec::new();
        let mut seeds = HashMap::new();
        let mut data = HashMap::new();
        for (resource, seed) in resources {
            let key = resource.path.trim_end_matches('/').to_string();
            definitions.push(resource);
            seeds.insert(key.clone(), seed.clone());
            data.insert(key, seed);
        }
        Self {
            definitions,
            seeds,
            data,
        }
    }

    fn reset(&mut self) -> usize {
        self.data = self.seeds.clone();
        self.definitions.len()
    }
}

async fn reload_workspace(
    workspace_root: &PathBuf,
    rules: &Arc<RwLock<Vec<MockRule>>>,
    resources: &Arc<RwLock<ResourceState>>,
    recording: Option<&RecordingState>,
    on_reload: &Option<Arc<dyn Fn(MockReload) + Send + Sync>>,
) {
    match (
        load_rules(workspace_root),
        load_resources(workspace_root),
        load_recording_config(workspace_root),
    ) {
        (Ok(next), Ok(next_resources), Ok(next_recording)) => {
            let mut guard = rules.write().await;
            let next_resources = ResourceState::from_loaded(next_resources);
            let mut resource_guard = resources.write().await;
            if let Some(recording) = recording {
                *recording.config.write().await = next_recording;
            }
            if *guard != next {
                let count = next.len();
                *guard = next;
                *resource_guard = next_resources;
                if let Some(callback) = on_reload {
                    callback(MockReload {
                        rules: count,
                        error: None,
                    });
                }
            } else if resource_guard.definitions != next_resources.definitions
                || resource_guard.seeds != next_resources.seeds
            {
                *resource_guard = next_resources;
            }
        }
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            if let Some(callback) = on_reload {
                callback(MockReload {
                    rules: rules.read().await.len(),
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

async fn wait_shutdown(rx: &mut Option<oneshot::Receiver<()>>) {
    match rx {
        Some(r) => {
            let _ = r.await;
        }
        None => std::future::pending::<()>().await,
    }
}

async fn handle(
    state: ServerState,
    req: Request<Incoming>,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
) -> Result<Response<MockResponseBody>, std::io::Error> {
    let path = req.uri().path().to_string();
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| path.clone());
    if path == "/__mirage" || path.starts_with("/__mirage/") {
        return Ok(handle_control(state, req).await);
    }

    let method = match map_method(req.method()) {
        Some(m) => m,
        None => return Ok(not_supported()),
    };

    let matched = {
        let rules = state.rules.read().await;
        find_match(&rules, method, &path).map(|Match { rule, params }| {
            (
                rule.name.clone(),
                rule.active_response().clone(),
                rule.fault.clone(),
                rule.passthrough,
                rule.upstream_url.clone(),
                rule.record,
                rule.condition_script.clone(),
                params,
            )
        })
    };
    if matched.is_none() {
        let resource_match = {
            let resources = state.resources.read().await;
            match_resource(&resources, method, &path)
        };
        if let Some(resource_match) = resource_match {
            return handle_resource(state, req, log, method, path, resource_match).await;
        }
    }
    let (status, body_bytes, headers, matched_name): (
        u16,
        Bytes,
        Vec<(String, String)>,
        Option<String>,
    ) = match matched {
        Some((
            rule_name,
            response,
            fault,
            passthrough,
            upstream_url,
            record,
            condition_script,
            params,
        )) => {
            if let Some(condition_script) = &condition_script {
                let condition = {
                    let runtime = ScriptRuntime::new();
                    runtime.eval_mock_condition(condition_script, method, &path, &params)
                };
                match condition {
                    Ok(true) => {}
                    Ok(false) => {
                        return finish_response(
                            state,
                            log,
                            method,
                            path,
                            404,
                            Bytes::from_static(b"{\"error\":\"no matching mock rule\"}"),
                            vec![("Content-Type".into(), "application/json".into())],
                            None,
                        )
                        .await;
                    }
                    Err(e) => {
                        return finish_response(
                            state,
                            log,
                            method,
                            path,
                            500,
                            Bytes::from(format!(r#"{{"error":"mock condition failed: {e}"}}"#)),
                            vec![("Content-Type".into(), "application/json".into())],
                            Some(rule_name),
                        )
                        .await;
                    }
                }
            }
            if passthrough {
                return proxy_request(
                    state,
                    log,
                    method,
                    path,
                    path_and_query,
                    req,
                    rule_name,
                    upstream_url,
                    record,
                )
                .await;
            }
            if let Some(fault) = &fault {
                match fault.kind() {
                    MockFaultKind::Timeout => {
                        std::future::pending::<()>().await;
                    }
                    MockFaultKind::PartialBody => {
                        return finish_partial_response(
                            state, log, method, path, rule_name, &response, &params,
                        )
                        .await;
                    }
                    MockFaultKind::ConnectionReset => {
                        let entry = MockLog {
                            method: format!("{method:?}").to_uppercase(),
                            path: path.clone(),
                            matched_rule: Some(rule_name.clone()),
                            status: 0,
                        };
                        record_log(&state.logs, entry.clone()).await;
                        log(entry);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::ConnectionReset,
                            "stubhouse injected connection reset",
                        ));
                    }
                    MockFaultKind::SlowResponse => {
                        tokio::time::sleep(Duration::from_millis(fault.delay_ms())).await;
                    }
                    MockFaultKind::Random5xx if should_inject_random_5xx(fault) => {
                        let status = random_5xx_status();
                        return finish_response(
                            state,
                            log,
                            method,
                            path,
                            status,
                            Bytes::from(format!(r#"{{"error":"injected {status}"}}"#)),
                            vec![("Content-Type".into(), "application/json".into())],
                            Some(rule_name),
                        )
                        .await;
                    }
                    MockFaultKind::Random5xx => {}
                }
            } else if response.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(response.delay_ms)).await;
            }
            response_tuple(rule_name, &response, method, &path, &params).await
        }
        None => (
            404,
            Bytes::from_static(b"{\"error\":\"no matching mock rule\"}"),
            vec![("Content-Type".into(), "application/json".into())],
            None,
        ),
    };

    finish_response(
        state,
        log,
        method,
        path,
        status,
        body_bytes,
        headers,
        matched_name,
    )
    .await
}

async fn finish_response(
    state: ServerState,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
    method: Method,
    path: String,
    status: u16,
    body_bytes: Bytes,
    headers: Vec<(String, String)>,
    matched_name: Option<String>,
) -> Result<Response<MockResponseBody>, std::io::Error> {
    let entry = MockLog {
        method: format!("{method:?}").to_uppercase(),
        path: path.clone(),
        matched_rule: matched_name,
        status,
    };
    record_log(&state.logs, entry.clone()).await;
    log(entry);

    let mut builder = Response::builder().status(status);
    for (k, v) in &headers {
        builder = builder.header(k, v);
    }
    builder
        .body(full_body(body_bytes))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

async fn finish_partial_response(
    state: ServerState,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
    method: Method,
    path: String,
    rule_name: String,
    response: &super::MockResponse,
    params: &std::collections::HashMap<String, String>,
) -> Result<Response<MockResponseBody>, std::io::Error> {
    let (status, body_bytes, headers, matched_name) =
        response_tuple(rule_name, response, method, &path, params).await;
    let entry = MockLog {
        method: format!("{method:?}").to_uppercase(),
        path,
        matched_rule: matched_name,
        status,
    };
    record_log(&state.logs, entry.clone()).await;
    log(entry);

    let midpoint = std::cmp::max(1, body_bytes.len() / 2);
    let partial = body_bytes.slice(..midpoint.min(body_bytes.len()));
    let mut builder = Response::builder().status(status);
    for (k, v) in &headers {
        builder = builder.header(k, v);
    }
    builder
        .header("Content-Length", body_bytes.len().to_string())
        .body(PartialBody::new(partial).boxed())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

fn full_body(bytes: Bytes) -> MockResponseBody {
    Full::new(bytes).map_err(|never| match never {}).boxed()
}

async fn response_tuple(
    rule_name: String,
    response: &super::MockResponse,
    method: Method,
    path: &str,
    params: &std::collections::HashMap<String, String>,
) -> (u16, Bytes, Vec<(String, String)>, Option<String>) {
    let (body, default_ct) = render_response_body(response, method, path, params);
    let mut headers = response.headers.clone();
    if let Some(ct) = default_ct {
        let already = headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        if !already {
            headers.push(("Content-Type".into(), ct));
        }
    }
    (response.status, body, headers, Some(rule_name))
}

fn should_inject_random_5xx(fault: &MockFault) -> bool {
    random_fraction() < fault.probability()
}

fn random_5xx_status() -> u16 {
    match (random_fraction() * 3.0) as u8 {
        0 => 500,
        1 => 502,
        _ => 503,
    }
}

fn random_fraction() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    f64::from(nanos % 10_000) / 10_000.0
}

async fn proxy_request(
    state: ServerState,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
    method: Method,
    path: String,
    path_and_query: String,
    req: Request<Incoming>,
    rule_name: String,
    upstream_url: Option<String>,
    record: bool,
) -> Result<Response<MockResponseBody>, std::io::Error> {
    let Some(upstream_url) = upstream_url else {
        return finish_response(
            state,
            log,
            method,
            path,
            502,
            Bytes::from_static(b"{\"error\":\"passthrough rule missing upstream_url\"}"),
            vec![("Content-Type".into(), "application/json".into())],
            Some(rule_name),
        )
        .await;
    };

    let target = match passthrough_target(&upstream_url, &path_and_query) {
        Ok(target) => target,
        Err(e) => {
            return finish_response(
                state,
                log,
                method,
                path,
                502,
                Bytes::from(format!(r#"{{"error":"invalid upstream_url: {e}"}}"#)),
                vec![("Content-Type".into(), "application/json".into())],
                Some(rule_name),
            )
            .await;
        }
    };

    let (parts, body) = req.into_parts();
    let request_headers = parts
        .headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let request_body = body
        .collect()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        .to_bytes();
    let reqwest_method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut builder = reqwest::Client::new()
        .request(reqwest_method, target)
        .body(request_body.to_vec());
    for (name, value) in parts.headers.iter() {
        if !is_hop_by_hop_header(name.as_str()) {
            builder = builder.header(name, value);
        }
    }

    let upstream = match builder.send().await {
        Ok(response) => response,
        Err(e) => {
            return finish_response(
                state,
                log,
                method,
                path,
                502,
                Bytes::from(format!(r#"{{"error":"passthrough failed: {e}"}}"#)),
                vec![("Content-Type".into(), "application/json".into())],
                Some(rule_name),
            )
            .await;
        }
    };

    let status = upstream.status().as_u16();
    let headers = upstream
        .headers()
        .iter()
        .filter(|(name, _)| !is_hop_by_hop_header(name.as_str()))
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let body = upstream
        .bytes()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if record {
        if let Some(recording) = &state.recording {
            let capture = RecordingCapture {
                rule_name: &rule_name,
                method,
                path: &path,
                request_headers: &request_headers,
                request_body: &request_body,
                status,
                response_headers: &headers,
                response_body: &body,
            };
            if let Err(e) = write_recording(recording, capture).await {
                eprintln!("stubhouse mock recording failed: {e}");
            }
        }
    }

    finish_response(
        state,
        log,
        method,
        path,
        status,
        body,
        headers,
        Some(rule_name),
    )
    .await
}

struct RecordingCapture<'a> {
    rule_name: &'a str,
    method: Method,
    path: &'a str,
    request_headers: &'a [(String, String)],
    request_body: &'a Bytes,
    status: u16,
    response_headers: &'a [(String, String)],
    response_body: &'a Bytes,
}

#[derive(serde::Serialize)]
struct RecordedRule {
    name: String,
    method: Method,
    path: String,
    response: super::MockResponse,
    recorded: RecordedMeta,
}

#[derive(serde::Serialize)]
struct RecordedMeta {
    source_rule: String,
    request: RecordedRequest,
}

#[derive(serde::Serialize)]
struct RecordedRequest {
    headers: Vec<(String, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
}

async fn write_recording(
    recording: &RecordingState,
    capture: RecordingCapture<'_>,
) -> Result<(), std::io::Error> {
    let config = recording.config.read().await.clone();
    let dir = if config.dir.is_absolute() {
        config.dir.clone()
    } else {
        recording.workspace_root.join(&config.dir)
    };
    fs::create_dir_all(&dir)?;

    let response_headers = scrub_headers(capture.response_headers, &config.scrub);
    let request_headers = scrub_headers(capture.request_headers, &config.scrub);
    let response_text = scrub_text(
        &String::from_utf8_lossy(capture.response_body),
        &config.scrub,
    );
    let request_text = if capture.request_body.is_empty() {
        None
    } else {
        Some(scrub_text(
            &String::from_utf8_lossy(capture.request_body),
            &config.scrub,
        ))
    };

    let rule = RecordedRule {
        name: format!("recorded-{}", slug_for_path(capture.path)),
        method: capture.method,
        path: capture.path.to_string(),
        response: super::MockResponse {
            status: capture.status,
            headers: response_headers,
            body: if serde_json::from_str::<serde_json::Value>(&response_text).is_ok() {
                MockBody::Json {
                    text: scrub_json_fields(&response_text, &config.scrub),
                }
            } else {
                MockBody::Text {
                    content_type: content_type(capture.response_headers)
                        .unwrap_or("text/plain; charset=utf-8")
                        .to_string(),
                    text: response_text,
                }
            },
            delay_ms: 0,
            body_script: None,
        },
        recorded: RecordedMeta {
            source_rule: capture.rule_name.to_string(),
            request: RecordedRequest {
                headers: request_headers,
                body: request_text,
            },
        },
    };

    let file = dir.join(format!(
        "{}-{}-{}.yaml",
        timestamp_millis(),
        method_label(capture.method).to_ascii_lowercase(),
        slug_for_path(capture.path)
    ));
    let yaml = serde_yaml::to_string(&rule)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(file, yaml)
}

fn scrub_headers(headers: &[(String, String)], scrub: &ScrubConfig) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            if scrub
                .headers
                .iter()
                .any(|header| header.eq_ignore_ascii_case(name))
            {
                (name.clone(), scrub.replacement.clone())
            } else {
                (name.clone(), scrub_text(value, scrub))
            }
        })
        .collect()
}

fn scrub_json_fields(text: &str, scrub: &ScrubConfig) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(text) else {
        return scrub_text(text, scrub);
    };
    redact_json_fields(&mut value, scrub);
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| scrub_text(text, scrub))
}

fn redact_json_fields(value: &mut serde_json::Value, scrub: &ScrubConfig) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if scrub
                    .json_fields
                    .iter()
                    .any(|field| field.eq_ignore_ascii_case(key))
                {
                    *value = serde_json::Value::String(scrub.replacement.clone());
                } else {
                    redact_json_fields(value, scrub);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                redact_json_fields(item, scrub);
            }
        }
        _ => {}
    }
}

fn scrub_text(text: &str, scrub: &ScrubConfig) -> String {
    scrub.text.iter().fold(text.to_string(), |value, pattern| {
        value.replace(pattern, &scrub.replacement)
    })
}

fn content_type(headers: &[(String, String)]) -> Option<&str> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.as_str())
}

fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn method_label(method: Method) -> &'static str {
    match method {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
    }
}

fn slug_for_path(path: &str) -> String {
    let slug = path
        .trim_matches('/')
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        "root".into()
    } else {
        slug
    }
}

fn passthrough_target(upstream_url: &str, path_and_query: &str) -> Result<String, url::ParseError> {
    let _ = url::Url::parse(upstream_url)?;
    Ok(format!(
        "{}/{}",
        upstream_url.trim_end_matches('/'),
        path_and_query.trim_start_matches('/')
    ))
}

fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
    )
}

async fn record_log(logs: &Arc<RwLock<Vec<MockLog>>>, entry: MockLog) {
    let mut guard = logs.write().await;
    guard.push(entry);
    if guard.len() > 200 {
        let overflow = guard.len() - 200;
        guard.drain(0..overflow);
    }
}

#[derive(Debug, Clone)]
struct ResourceMatch {
    resource: MockResource,
    key: String,
    id: Option<String>,
}

fn match_resource(state: &ResourceState, method: Method, path: &str) -> Option<ResourceMatch> {
    for resource in &state.definitions {
        if !resource.auto_crud {
            continue;
        }
        let key = resource.path.trim_end_matches('/').to_string();
        let path = path.trim_end_matches('/');
        if path == key && matches!(method, Method::Get | Method::Post) {
            return Some(ResourceMatch {
                resource: resource.clone(),
                key,
                id: None,
            });
        }
        let item_prefix = format!("{key}/");
        if let Some(id) = path.strip_prefix(&item_prefix) {
            if !id.is_empty()
                && !id.contains('/')
                && matches!(
                    method,
                    Method::Get | Method::Put | Method::Patch | Method::Delete
                )
            {
                return Some(ResourceMatch {
                    resource: resource.clone(),
                    key,
                    id: Some(id.to_string()),
                });
            }
        }
    }
    None
}

async fn handle_resource(
    state: ServerState,
    req: Request<Incoming>,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
    method: Method,
    path: String,
    resource_match: ResourceMatch,
) -> Result<Response<MockResponseBody>, std::io::Error> {
    let matched_name = Some(format!("resource:{}", resource_match.key));
    let response = match method {
        Method::Get if resource_match.id.is_none() => {
            let resources = state.resources.read().await;
            let items = resources
                .data
                .get(&resource_match.key)
                .cloned()
                .unwrap_or_default();
            json_bytes_response(200, &items)
        }
        Method::Get => {
            let resources = state.resources.read().await;
            match find_resource_item(
                resources.data.get(&resource_match.key),
                &resource_match.resource.id_field,
                resource_match.id.as_deref().unwrap_or_default(),
            ) {
                Some(item) => json_bytes_response(200, item),
                None => json_bytes_response(404, &serde_json::json!({ "error": "not found" })),
            }
        }
        Method::Post => {
            let mut item = parse_resource_body(req).await?;
            let mut resources = state.resources.write().await;
            let items = resources
                .data
                .entry(resource_match.key.clone())
                .or_default();
            ensure_resource_id(&mut item, &resource_match.resource.id_field, || {
                next_resource_id(Some(items.as_slice()), &resource_match.resource.id_field)
            });
            items.push(item.clone());
            json_bytes_response(201, &item)
        }
        Method::Put => {
            let mut item = parse_resource_body(req).await?;
            if let Some(id) = &resource_match.id {
                set_resource_id(&mut item, &resource_match.resource.id_field, id);
            }
            let mut resources = state.resources.write().await;
            let items = resources
                .data
                .entry(resource_match.key.clone())
                .or_default();
            let id = resource_match.id.as_deref().unwrap_or_default();
            if let Some(existing) =
                find_resource_item_mut(items, &resource_match.resource.id_field, id)
            {
                *existing = item.clone();
            } else {
                items.push(item.clone());
            }
            json_bytes_response(200, &item)
        }
        Method::Patch => {
            let patch = parse_resource_body(req).await?;
            let mut resources = state.resources.write().await;
            let items = resources
                .data
                .entry(resource_match.key.clone())
                .or_default();
            let id = resource_match.id.as_deref().unwrap_or_default();
            match find_resource_item_mut(items, &resource_match.resource.id_field, id) {
                Some(existing) => {
                    merge_json(existing, patch);
                    json_bytes_response(200, existing)
                }
                None => json_bytes_response(404, &serde_json::json!({ "error": "not found" })),
            }
        }
        Method::Delete => {
            let mut resources = state.resources.write().await;
            let items = resources
                .data
                .entry(resource_match.key.clone())
                .or_default();
            let id = resource_match.id.as_deref().unwrap_or_default();
            let before = items.len();
            items.retain(|item| !resource_id_matches(item, &resource_match.resource.id_field, id));
            if items.len() == before {
                json_bytes_response(404, &serde_json::json!({ "error": "not found" }))
            } else {
                (204, Bytes::new())
            }
        }
        _ => json_bytes_response(405, &serde_json::json!({ "error": "method not allowed" })),
    };

    finish_response(
        state,
        log,
        method,
        path,
        response.0,
        response.1,
        vec![("Content-Type".into(), "application/json".into())],
        matched_name,
    )
    .await
}

async fn parse_resource_body(req: Request<Incoming>) -> Result<serde_json::Value, std::io::Error> {
    let bytes = req
        .into_body()
        .collect()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        .to_bytes();
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn json_bytes_response<T: serde::Serialize + ?Sized>(status: u16, value: &T) -> (u16, Bytes) {
    match serde_json::to_vec(value) {
        Ok(body) => (status, Bytes::from(body)),
        Err(_) => (
            500,
            Bytes::from_static(b"{\"error\":\"serialize response\"}"),
        ),
    }
}

fn find_resource_item<'a>(
    items: Option<&'a Vec<serde_json::Value>>,
    id_field: &str,
    id: &str,
) -> Option<&'a serde_json::Value> {
    items?
        .iter()
        .find(|item| resource_id_matches(item, id_field, id))
}

fn find_resource_item_mut<'a>(
    items: &'a mut [serde_json::Value],
    id_field: &str,
    id: &str,
) -> Option<&'a mut serde_json::Value> {
    items
        .iter_mut()
        .find(|item| resource_id_matches(item, id_field, id))
}

fn resource_id_matches(item: &serde_json::Value, id_field: &str, id: &str) -> bool {
    match item.get(id_field) {
        Some(serde_json::Value::String(value)) => value == id,
        Some(serde_json::Value::Number(value)) => value.to_string() == id,
        Some(value) => value.to_string().trim_matches('"') == id,
        None => false,
    }
}

fn ensure_resource_id(
    item: &mut serde_json::Value,
    id_field: &str,
    next_id: impl FnOnce() -> serde_json::Value,
) {
    if item.get(id_field).is_none() {
        set_resource_id_value(item, id_field, next_id());
    }
}

fn set_resource_id(item: &mut serde_json::Value, id_field: &str, id: &str) {
    set_resource_id_value(item, id_field, serde_json::Value::String(id.to_string()));
}

fn set_resource_id_value(item: &mut serde_json::Value, id_field: &str, id: serde_json::Value) {
    if let serde_json::Value::Object(map) = item {
        map.insert(id_field.to_string(), id);
    }
}

fn next_resource_id(items: Option<&[serde_json::Value]>, id_field: &str) -> serde_json::Value {
    let next = items
        .unwrap_or_default()
        .iter()
        .filter_map(|item| item.get(id_field)?.as_u64())
        .max()
        .unwrap_or(0)
        + 1;
    serde_json::Value::Number(next.into())
}

fn merge_json(target: &mut serde_json::Value, patch: serde_json::Value) {
    match (target, patch) {
        (serde_json::Value::Object(target), serde_json::Value::Object(patch)) => {
            for (key, value) in patch {
                target.insert(key, value);
            }
        }
        (target, patch) => *target = patch,
    }
}

#[derive(Debug, serde::Serialize)]
struct ControlStatus {
    ok: bool,
    rules: usize,
    scenarios: Vec<ControlScenario>,
}

#[derive(Debug, serde::Serialize)]
struct ControlScenario {
    name: String,
    rules: usize,
    active_rules: usize,
}

#[derive(Debug, serde::Deserialize)]
struct ScenarioSwitch {
    scenario: Option<String>,
    name: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ResetResponse {
    ok: bool,
    reset: bool,
    resources: usize,
}

async fn handle_control(state: ServerState, req: Request<Incoming>) -> Response<MockResponseBody> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    match (method, path.as_str()) {
        (hyper::Method::GET, "/__mirage/status") => {
            let rules = state.rules.read().await;
            json_response(200, &control_status(&rules))
        }
        (hyper::Method::GET, "/__mirage/rules") => {
            let rules = state.rules.read().await;
            json_response(200, &*rules)
        }
        (hyper::Method::GET, "/__mirage/log") => {
            let limit = req
                .uri()
                .query()
                .and_then(|query| {
                    query
                        .split('&')
                        .find_map(|part| part.strip_prefix("limit="))
                })
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100);
            let logs = state.logs.read().await;
            let start = logs.len().saturating_sub(limit);
            json_response(200, &logs[start..])
        }
        (hyper::Method::POST, "/__mirage/scenario") => switch_scenario(state, req).await,
        (hyper::Method::POST, "/__mirage/reset") => {
            let resources = state.resources.write().await.reset();
            json_response(
                200,
                &ResetResponse {
                    ok: true,
                    reset: true,
                    resources,
                },
            )
        }
        (hyper::Method::GET | hyper::Method::POST, _) => {
            json_error(404, "unknown __mirage endpoint")
        }
        _ => json_error(405, "method not allowed for __mirage endpoint"),
    }
}

async fn switch_scenario(state: ServerState, req: Request<Incoming>) -> Response<MockResponseBody> {
    let bytes = match req.into_body().collect().await {
        Ok(body) => body.to_bytes(),
        Err(e) => return json_error(400, &format!("read request body: {e}")),
    };
    let payload: ScenarioSwitch = match serde_json::from_slice(&bytes) {
        Ok(payload) => payload,
        Err(e) => return json_error(400, &format!("parse json body: {e}")),
    };
    let Some(name) = payload.scenario.or(payload.name) else {
        return json_error(400, "missing scenario");
    };

    let mut rules = state.rules.write().await;
    let mut rules_changed = 0usize;
    for rule in rules.iter_mut() {
        if !rule.scenarios.iter().any(|scenario| scenario.name == name) {
            continue;
        }

        let before = rule.scenarios.clone();
        for scenario in &mut rule.scenarios {
            scenario.active = scenario.name == name;
        }
        if rule.scenarios != before {
            rules_changed += 1;
        }
    }

    json_response(
        200,
        &ScenarioActivation {
            scenario: name,
            files_changed: 0,
            rules_changed,
        },
    )
}

fn control_status(rules: &[MockRule]) -> ControlStatus {
    let mut scenarios = std::collections::BTreeMap::<String, (usize, usize)>::new();
    for rule in rules {
        for scenario in &rule.scenarios {
            let entry = scenarios.entry(scenario.name.clone()).or_default();
            entry.0 += 1;
            if scenario.active {
                entry.1 += 1;
            }
        }
    }

    ControlStatus {
        ok: true,
        rules: rules.len(),
        scenarios: scenarios
            .into_iter()
            .map(|(name, (rules, active_rules))| ControlScenario {
                name,
                rules,
                active_rules,
            })
            .collect(),
    }
}

fn json_response<T: serde::Serialize + ?Sized>(
    status: u16,
    value: &T,
) -> Response<MockResponseBody> {
    match serde_json::to_vec(value) {
        Ok(body) => Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(full_body(Bytes::from(body)))
            .unwrap_or_else(|_| internal_error()),
        Err(_) => internal_error(),
    }
}

fn json_error(status: u16, message: &str) -> Response<MockResponseBody> {
    json_response(status, &serde_json::json!({ "error": message }))
}

fn render_response_body(
    response: &super::MockResponse,
    method: Method,
    path: &str,
    params: &std::collections::HashMap<String, String>,
) -> (Bytes, Option<String>) {
    if let Some(script) = &response.body_script {
        return match ScriptRuntime::new().render_mock_body(script, method, path, params) {
            Ok(rendered) => (
                Bytes::from(rendered.into_bytes()),
                Some("application/json".into()),
            ),
            Err(e) => (
                Bytes::from(format!(r#"{{"error":"mock body script failed: {e}"}}"#)),
                Some("application/json".into()),
            ),
        };
    }

    match &response.body {
        MockBody::None => (Bytes::new(), None),
        MockBody::Text { content_type, text } => {
            let rendered = render_body(text, params);
            (
                Bytes::from(rendered.into_bytes()),
                Some(content_type.clone()),
            )
        }
        MockBody::Json { text } => {
            let rendered = render_body(text, params);
            (
                Bytes::from(rendered.into_bytes()),
                Some("application/json".into()),
            )
        }
    }
}

fn map_method(m: &hyper::Method) -> Option<Method> {
    use hyper::Method as H;
    Some(match *m {
        H::GET => Method::Get,
        H::POST => Method::Post,
        H::PUT => Method::Put,
        H::PATCH => Method::Patch,
        H::DELETE => Method::Delete,
        H::HEAD => Method::Head,
        H::OPTIONS => Method::Options,
        _ => return None,
    })
}

fn not_supported() -> Response<MockResponseBody> {
    Response::builder()
        .status(405)
        .header("Content-Type", "application/json")
        .body(full_body(Bytes::from_static(
            b"{\"error\":\"method not supported by mock server\"}",
        )))
        .unwrap()
}

fn internal_error() -> Response<MockResponseBody> {
    Response::builder()
        .status(500)
        .body(full_body(Bytes::from_static(b"internal error")))
        .unwrap()
}

struct PartialBody {
    chunk: Option<Bytes>,
    failed: bool,
}

impl PartialBody {
    fn new(chunk: Bytes) -> Self {
        Self {
            chunk: Some(chunk),
            failed: false,
        }
    }
}

impl HyperBody for PartialBody {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if let Some(chunk) = self.chunk.take() {
            return Poll::Ready(Some(Ok(Frame::data(chunk))));
        }
        if !self.failed {
            self.failed = true;
            return Poll::Ready(Some(Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "stubhouse injected partial body",
            )))));
        }
        Poll::Ready(None)
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockBody, MockFaultConfig, MockFaultKind, MockResponse};
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn serves_static_json_and_interpolates_params() {
        let rules = vec![MockRule {
            name: "get-user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Json {
                    text: r#"{"id":"{{params.id}}","name":"Alice"}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener); // free the port; run() will rebind

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });

        // Give the server a tick to bind.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.headers()["content-type"], "application/json");
        let body = resp.text().await.unwrap();
        assert_eq!(body, r#"{"id":"42","name":"Alice"}"#);

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn returns_404_when_no_rule_matches() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(vec![], bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/anything"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn text_body_sets_configured_content_type() {
        let rules = vec![MockRule {
            name: "robots".into(),
            method: Method::Get,
            path: "/robots.txt".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Text {
                    content_type: "text/plain; charset=utf-8".into(),
                    text: "User-agent: *".into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/robots.txt"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.headers()["content-type"], "text/plain; charset=utf-8");
        assert_eq!(resp.text().await.unwrap(), "User-agent: *");

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn serves_active_scenario_response() {
        let rules = vec![MockRule {
            name: "get-user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Json {
                    text: r#"{"id":"{{params.id}}","state":"default"}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![super::super::MockScenario {
                name: "missing".into(),
                active: true,
                response: MockResponse {
                    status: 404,
                    headers: vec![],
                    body: MockBody::Json {
                        text: r#"{"id":"{{params.id}}","error":"missing"}"#.into(),
                    },
                    delay_ms: 0,
                    body_script: None,
                },
            }],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);
        assert_eq!(
            resp.text().await.unwrap(),
            r#"{"id":"42","error":"missing"}"#
        );

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn hot_reload_serves_updated_rule_file() {
        let dir = TempDir::new().unwrap();
        let rule_path = dir.path().join("collections/users/mocks/get.yaml");
        fs::create_dir_all(rule_path.parent().unwrap()).unwrap();
        fs::write(
            &rule_path,
            r#"
name: get-user
method: GET
path: /users/:id
response:
  status: 200
  body:
    kind: json
    text: '{"version":1}'
"#,
        )
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let root = dir.path().to_path_buf();
        let handle =
            tokio::spawn(
                async move { run_with_hot_reload(root, bound, Some(rx), None, None).await },
            );
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.text().await.unwrap(), r#"{"version":1}"#);

        fs::write(
            &rule_path,
            r#"
name: get-user
method: GET
path: /users/:id
response:
  status: 202
  body:
    kind: json
    text: '{"version":2}'
"#,
        )
        .unwrap();

        let mut updated = None;
        for _ in 0..20 {
            let resp = reqwest::get(format!("http://{bound}/users/42"))
                .await
                .unwrap();
            if resp.status() == 202 {
                updated = Some(resp.text().await.unwrap());
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        assert_eq!(updated.as_deref(), Some(r#"{"version":2}"#));

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn control_api_reports_status_rules_and_log() {
        let rules = vec![MockRule {
            name: "get-user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Json {
                    text: r#"{"id":"{{params.id}}"}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![super::super::MockScenario {
                name: "missing".into(),
                active: false,
                response: MockResponse {
                    status: 404,
                    headers: vec![],
                    body: MockBody::None,
                    delay_ms: 0,
                    body_script: None,
                },
            }],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let status_text = client
            .get(format!("http://{bound}/__mirage/status"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let status: serde_json::Value = serde_json::from_str(&status_text).unwrap();
        assert_eq!(status["ok"], true);
        assert_eq!(status["rules"], 1);
        assert_eq!(status["scenarios"][0]["name"], "missing");

        let rules_text = client
            .get(format!("http://{bound}/__mirage/rules"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let rules: serde_json::Value = serde_json::from_str(&rules_text).unwrap();
        assert_eq!(rules[0]["name"], "get-user");

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        let logs_text = client
            .get(format!("http://{bound}/__mirage/log?limit=1"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let logs: serde_json::Value = serde_json::from_str(&logs_text).unwrap();
        assert_eq!(logs[0]["method"], "GET");
        assert_eq!(logs[0]["path"], "/users/42");
        assert_eq!(logs[0]["matched_rule"], "get-user");

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn control_api_switches_scenario_in_memory() {
        let rules = vec![MockRule {
            name: "get-user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Json {
                    text: r#"{"state":"default"}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![super::super::MockScenario {
                name: "missing".into(),
                active: false,
                response: MockResponse {
                    status: 404,
                    headers: vec![],
                    body: MockBody::Json {
                        text: r#"{"state":"missing"}"#.into(),
                    },
                    delay_ms: 0,
                    body_script: None,
                },
            }],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("http://{bound}/__mirage/scenario"))
            .header("Content-Type", "application/json")
            .body(r#"{"scenario":"missing"}"#)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let activation_text = resp.text().await.unwrap();
        let activation: serde_json::Value = serde_json::from_str(&activation_text).unwrap();
        assert_eq!(activation["scenario"], "missing");
        assert_eq!(activation["files_changed"], 0);
        assert_eq!(activation["rules_changed"], 1);

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);
        assert_eq!(resp.text().await.unwrap(), r#"{"state":"missing"}"#);

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn random_5xx_fault_can_replace_response() {
        let rules = vec![MockRule {
            name: "flaky".into(),
            method: Method::Get,
            path: "/flaky".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Json {
                    text: r#"{"ok":true}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: Some(super::super::MockFault::Config(MockFaultConfig {
                kind: MockFaultKind::Random5xx,
                delay_ms: None,
                probability: Some(1.0),
            })),
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/flaky")).await.unwrap();
        assert!([500, 502, 503].contains(&resp.status().as_u16()));

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn slow_response_fault_delays_response() {
        let rules = vec![MockRule {
            name: "slow".into(),
            method: Method::Get,
            path: "/slow".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Text {
                    content_type: "text/plain".into(),
                    text: "ok".into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: Some(super::super::MockFault::Config(MockFaultConfig {
                kind: MockFaultKind::SlowResponse,
                delay_ms: Some(40),
                probability: None,
            })),
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let started = std::time::Instant::now();
        let resp = reqwest::get(format!("http://{bound}/slow")).await.unwrap();
        assert_eq!(resp.status(), 200);
        assert!(started.elapsed() >= Duration::from_millis(35));

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn connection_reset_fault_closes_connection() {
        let rules = vec![MockRule {
            name: "reset".into(),
            method: Method::Get,
            path: "/reset".into(),
            priority: 0,
            response: MockResponse::default(),
            scenarios: vec![],
            fault: Some(super::super::MockFault::Kind(
                MockFaultKind::ConnectionReset,
            )),
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let err = reqwest::get(format!("http://{bound}/reset"))
            .await
            .unwrap_err();
        assert!(err.is_request() || err.is_body() || err.is_decode());

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn partial_body_fault_closes_connection() {
        let rules = vec![MockRule {
            name: "partial".into(),
            method: Method::Get,
            path: "/partial".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::Text {
                    content_type: "text/plain".into(),
                    text: "abcdef".into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: Some(super::super::MockFault::Kind(MockFaultKind::PartialBody)),
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let err = reqwest::get(format!("http://{bound}/partial"))
            .await
            .unwrap_err();
        assert!(err.is_request() || err.is_body() || err.is_decode());

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn timeout_fault_never_responds() {
        let rules = vec![MockRule {
            name: "timeout".into(),
            method: Method::Get,
            path: "/timeout".into(),
            priority: 0,
            response: MockResponse::default(),
            scenarios: vec![],
            fault: Some(super::super::MockFault::Kind(MockFaultKind::Timeout)),
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(80))
            .build()
            .unwrap();
        let err = client
            .get(format!("http://{bound}/timeout"))
            .send()
            .await
            .unwrap_err();
        assert!(err.is_timeout());

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn passthrough_rule_proxies_to_upstream() {
        let upstream_rules = vec![MockRule {
            name: "upstream".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 201,
                headers: vec![("X-Upstream".into(), "yes".into())],
                body: MockBody::Json {
                    text: r#"{"id":"{{params.id}}","source":"upstream"}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_addr = upstream_listener.local_addr().unwrap();
        drop(upstream_listener);
        let (upstream_tx, upstream_rx) = oneshot::channel();
        let upstream_handle = tokio::spawn(async move {
            run(upstream_rules, upstream_addr, Some(upstream_rx), None).await
        });

        let rules = vec![MockRule {
            name: "proxy-user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse::default(),
            scenarios: vec![],
            fault: None,
            passthrough: true,
            upstream_url: Some(format!("http://{upstream_addr}")),
            record: false,
            condition_script: None,
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/users/42?include=profile"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
        assert_eq!(resp.headers()["x-upstream"], "yes");
        assert_eq!(
            resp.text().await.unwrap(),
            r#"{"id":"42","source":"upstream"}"#
        );

        let _ = tx.send(());
        let _ = upstream_tx.send(());
        let _ = handle.await;
        let _ = upstream_handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recording_mode_captures_passthrough_with_scrubbing() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("collections/api/mocks")).unwrap();
        fs::write(
            dir.path().join("workspace.yaml"),
            r#"
name: demo
version: "1"
recording:
  dir: recordings
  scrub:
    headers:
      - Authorization
    json_fields:
      - token
    text:
      - secret-token
"#,
        )
        .unwrap();

        let upstream_rules = vec![MockRule {
            name: "upstream-token".into(),
            method: Method::Get,
            path: "/token".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![("X-Upstream".into(), "yes".into())],
                body: MockBody::Json {
                    text: r#"{"token":"secret-token","ok":true}"#.into(),
                },
                delay_ms: 0,
                body_script: None,
            },
            scenarios: vec![],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: None,
        }];
        let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_addr = upstream_listener.local_addr().unwrap();
        drop(upstream_listener);
        let (upstream_tx, upstream_rx) = oneshot::channel();
        let upstream_handle = tokio::spawn(async move {
            run(upstream_rules, upstream_addr, Some(upstream_rx), None).await
        });

        fs::write(
            dir.path().join("collections/api/mocks/token.yaml"),
            format!(
                r#"
name: proxy-token
method: GET
path: /token
passthrough: true
upstream_url: http://{upstream_addr}
record: true
"#
            ),
        )
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let root = dir.path().to_path_buf();
        let handle =
            tokio::spawn(
                async move { run_with_hot_reload(root, bound, Some(rx), None, None).await },
            );
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::Client::new()
            .get(format!("http://{bound}/token"))
            .header("Authorization", "Bearer secret-token")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(
            resp.text().await.unwrap(),
            r#"{"token":"secret-token","ok":true}"#
        );

        let recordings_dir = dir.path().join("recordings");
        let mut recording = None;
        for _ in 0..20 {
            if recordings_dir.exists() {
                let files = fs::read_dir(&recordings_dir)
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                if let Some(file) = files.first() {
                    recording = Some(fs::read_to_string(file.path()).unwrap());
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let recording = recording.expect("recording file should be written");
        assert!(recording.contains("proxy-token"));
        assert!(recording.contains("authorization"));
        assert!(recording.contains("[REDACTED]"));
        assert!(!recording.contains("secret-token"));

        let _ = tx.send(());
        let _ = upstream_tx.send(());
        let _ = handle.await;
        let _ = upstream_handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mock_condition_and_body_generator_scripts_are_applied() {
        let rules = vec![MockRule {
            name: "scripted".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::None,
                delay_ms: 0,
                body_script: Some(r#"`{"id":"${request.params["id"]}","scripted":true}`"#.into()),
            },
            scenarios: vec![],
            fault: None,
            passthrough: false,
            upstream_url: None,
            record: false,
            condition_script: Some(r#"request.params["id"] == "42""#.into()),
        }];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(async move { run(rules, bound, Some(rx), None).await });
        tokio::time::sleep(Duration::from_millis(50)).await;

        let resp = reqwest::get(format!("http://{bound}/users/42"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.text().await.unwrap(), r#"{"id":"42","scripted":true}"#);

        let resp = reqwest::get(format!("http://{bound}/users/7"))
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);

        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mock_resources_provide_stateful_crud_and_reset() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("fixtures")).unwrap();
        fs::write(
            dir.path().join("workspace.yaml"),
            r#"
name: demo
version: "1"
mock_resources:
  - path: /users
    id_field: id
    seed_file: fixtures/users.yaml
    auto_crud: true
"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("fixtures/users.yaml"),
            r#"
- id: 1
  name: Alice
- id: 2
  name: Ben
"#,
        )
        .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound = listener.local_addr().unwrap();
        drop(listener);

        let (tx, rx) = oneshot::channel();
        let root = dir.path().to_path_buf();
        let handle =
            tokio::spawn(
                async move { run_with_hot_reload(root, bound, Some(rx), None, None).await },
            );
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = reqwest::Client::new();
        let list: serde_json::Value = serde_json::from_str(
            &client
                .get(format!("http://{bound}/users"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(list.as_array().unwrap().len(), 2);

        let created: serde_json::Value = serde_json::from_str(
            &client
                .post(format!("http://{bound}/users"))
                .header("Content-Type", "application/json")
                .body(r#"{"name":"Casey"}"#)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(created["id"], 3);

        let patched: serde_json::Value = serde_json::from_str(
            &client
                .patch(format!("http://{bound}/users/3"))
                .header("Content-Type", "application/json")
                .body(r#"{"name":"C"}"#)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(patched["name"], "C");

        let reset: serde_json::Value = serde_json::from_str(
            &client
                .post(format!("http://{bound}/__mirage/reset"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(reset["resources"], 1);

        let resp = client
            .get(format!("http://{bound}/users/3"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 404);

        let _ = tx.send(());
        let _ = handle.await;
    }
}
