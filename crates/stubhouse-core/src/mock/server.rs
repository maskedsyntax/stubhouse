//! Embedded HTTP server that serves mock rules.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::matcher::{find_match, Match};
use super::{render_body, MockBody, MockError, MockRule};
use crate::http::Method;

/// Wire-format payload for the server's request log channel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MockLog {
    pub method: String,
    pub path: String,
    pub matched_rule: Option<String>,
    pub status: u16,
}

/// Run the mock server in the foreground, until ctrl-c (or `shutdown_rx` fires).
/// On bind error, returns immediately.
pub async fn run(
    rules: Vec<MockRule>,
    addr: SocketAddr,
    shutdown_rx: Option<oneshot::Receiver<()>>,
    on_log: Option<Arc<dyn Fn(MockLog) + Send + Sync>>,
) -> Result<(), MockError> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| MockError::Server(format!("bind {addr}: {e}")))?;

    let rules = Arc::new(rules);
    let log = on_log.unwrap_or_else(|| Arc::new(|_| {}));

    let mut shutdown = shutdown_rx;
    loop {
        let accept = listener.accept();
        let conn = tokio::select! {
            res = accept => res,
            _ = wait_shutdown(&mut shutdown) => return Ok(()),
        };

        let (stream, _peer) = match conn {
            Ok(v) => v,
            Err(e) => {
                eprintln!("stubhouse mock: accept error: {e}");
                continue;
            }
        };

        let rules = Arc::clone(&rules);
        let log = Arc::clone(&log);
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let rules = Arc::clone(&rules);
                let log = Arc::clone(&log);
                async move { Ok::<_, std::convert::Infallible>(handle(rules, req, log).await) }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("stubhouse mock: connection error: {e}");
            }
        });
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
    rules: Arc<Vec<MockRule>>,
    req: Request<Incoming>,
    log: Arc<dyn Fn(MockLog) + Send + Sync>,
) -> Response<Full<Bytes>> {
    let method = match map_method(req.method()) {
        Some(m) => m,
        None => return not_supported(),
    };
    let path = req.uri().path().to_string();

    let m = find_match(&rules, method, &path);
    let (status, body_bytes, headers, matched_name): (
        u16,
        Bytes,
        Vec<(String, String)>,
        Option<String>,
    ) = match m {
        Some(Match { rule, params }) => {
            let response = rule.active_response();
            if response.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(response.delay_ms)).await;
            }
            let (body, default_ct) = render_response_body(&response.body, &params);
            let mut headers = response.headers.clone();
            if let Some(ct) = default_ct {
                let already = headers
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
                if !already {
                    headers.push(("Content-Type".into(), ct));
                }
            }
            (response.status, body, headers, Some(rule.name.clone()))
        }
        None => (
            404,
            Bytes::from_static(b"{\"error\":\"no matching mock rule\"}"),
            vec![("Content-Type".into(), "application/json".into())],
            None,
        ),
    };

    log(MockLog {
        method: format!("{method:?}").to_uppercase(),
        path: path.clone(),
        matched_rule: matched_name,
        status,
    });

    let mut builder = Response::builder().status(status);
    for (k, v) in &headers {
        builder = builder.header(k, v);
    }
    builder
        .body(Full::new(body_bytes))
        .unwrap_or_else(|_| internal_error())
}

fn render_response_body(
    body: &MockBody,
    params: &std::collections::HashMap<String, String>,
) -> (Bytes, Option<String>) {
    match body {
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

fn not_supported() -> Response<Full<Bytes>> {
    Response::builder()
        .status(405)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from_static(
            b"{\"error\":\"method not supported by mock server\"}",
        )))
        .unwrap()
}

fn internal_error() -> Response<Full<Bytes>> {
    Response::builder()
        .status(500)
        .body(Full::new(Bytes::from_static(b"internal error")))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockBody, MockResponse};

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
            },
            scenarios: vec![],
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
            },
            scenarios: vec![],
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
                },
            }],
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
}
