#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use serde::Serialize;
use stubhouse_core::{
    from_bruno_bru, from_har, from_insomnia_v4, from_postman_v21, interpolate_compose,
    list_environments, load_environment,
    mock::{
        activate_scenario, list_scenarios, load_rules,
        server::{run_with_hot_reload, MockLog, MockReload},
        ScenarioActivation, ScenarioEntry,
    },
    run_workspace_tests, save_environment, send, to_curl, Compose, Environment, EnvironmentEntry,
    EnvironmentFile, History, HistoryEntry, ImportedRequest, RequestDefinition, RequestEntry,
    Response, TestRunResult, Workspace, WorkspaceManifest,
};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
struct AppState {
    workspace: Mutex<Option<Workspace>>,
    history: Mutex<Option<History>>,
    active_env: Mutex<Option<Environment>>,
    mock_server: Mutex<Option<MockServerRuntime>>,
    send_seq: AtomicU64,
    send_results: Mutex<HashMap<u64, Result<ResponseDto, String>>>,
}

struct MockServerRuntime {
    bind: String,
    port: u16,
    rules: Arc<AtomicUsize>,
    logs: Arc<Mutex<Vec<MockLog>>>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseDto {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
    elapsed_ms: u64,
    size_bytes: usize,
    history_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct SendResponseEvent {
    request_id: u64,
    response: ResponseDto,
}

#[derive(Debug, Clone, Serialize)]
struct AsyncSendResult {
    done: bool,
    response: Option<ResponseDto>,
    error: Option<String>,
}

impl ResponseDto {
    fn from_response(r: Response, history_id: Option<i64>) -> Self {
        Self {
            status: r.status,
            headers: r.headers,
            body: String::from_utf8_lossy(&r.body).into_owned(),
            elapsed_ms: r.elapsed_ms,
            size_bytes: r.size_bytes,
            history_id,
        }
    }
}

#[derive(Debug, Serialize)]
struct WorkspaceInfo {
    root: PathBuf,
    manifest: WorkspaceManifest,
}

#[derive(Debug, Serialize)]
struct HistoryReplay {
    request: Compose,
    response: ResponseDto,
}

#[derive(Debug, Serialize)]
struct MockServerStatus {
    running: bool,
    bind: String,
    port: u16,
    url: String,
    rules: usize,
    logs: Vec<MockLog>,
}

impl MockServerStatus {
    fn stopped() -> Self {
        Self {
            running: false,
            bind: "127.0.0.1".into(),
            port: 4000,
            url: "http://127.0.0.1:4000".into(),
            rules: 0,
            logs: vec![],
        }
    }
}

#[tauri::command]
async fn send_request(
    req: Compose,
    request_id: Option<u64>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ResponseDto, String> {
    let resolved = {
        let guard = state.active_env.lock().unwrap();
        match guard.as_ref() {
            Some(env) => interpolate_compose(&req, &env.variables),
            None => req.clone(),
        }
    };
    let wire = resolved.clone().build().map_err(|e| e.to_string())?;
    let resp = tokio::time::timeout(Duration::from_secs(30), send(wire))
        .await
        .map_err(|_| "request timed out after 30 seconds".to_string())?
        .map_err(|e| e.to_string())?;

    let history_id = state
        .history
        .try_lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|h| h.record(&resolved, &resp).ok()));

    let dto = ResponseDto::from_response(resp, history_id);
    if let Some(request_id) = request_id {
        let _ = app.emit(
            "send-response",
            SendResponseEvent {
                request_id,
                response: dto.clone(),
            },
        );
    }
    eprintln!(
        "stubhouse app: returning response {} (history: {})",
        dto.status,
        history_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "skipped".into())
    );
    Ok(dto)
}

#[tauri::command]
fn start_send_request(req: Compose, state: State<'_, AppState>, app: AppHandle) -> u64 {
    let request_id = state.send_seq.fetch_add(1, Ordering::Relaxed) + 1;
    {
        let mut results = state.send_results.lock().unwrap();
        results.remove(&request_id);
    }
    tauri::async_runtime::spawn(async move {
        let result = perform_send_request(req, &app).await;
        let state = app.state::<AppState>();
        state
            .send_results
            .lock()
            .unwrap()
            .insert(request_id, result);
    });
    request_id
}

#[tauri::command]
fn poll_send_result(request_id: u64, state: State<'_, AppState>) -> AsyncSendResult {
    let mut results = state.send_results.lock().unwrap();
    match results.remove(&request_id) {
        Some(Ok(response)) => AsyncSendResult {
            done: true,
            response: Some(response),
            error: None,
        },
        Some(Err(error)) => AsyncSendResult {
            done: true,
            response: None,
            error: Some(error),
        },
        None => AsyncSendResult {
            done: false,
            response: None,
            error: None,
        },
    }
}

async fn perform_send_request(req: Compose, app: &AppHandle) -> Result<ResponseDto, String> {
    let state = app.state::<AppState>();
    let resolved = {
        let guard = state.active_env.lock().unwrap();
        match guard.as_ref() {
            Some(env) => interpolate_compose(&req, &env.variables),
            None => req.clone(),
        }
    };
    let wire = resolved.clone().build().map_err(|e| e.to_string())?;
    let resp = tokio::time::timeout(Duration::from_secs(30), send(wire))
        .await
        .map_err(|_| "request timed out after 30 seconds".to_string())?
        .map_err(|e| e.to_string())?;

    let history_id = state
        .history
        .try_lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|h| h.record(&resolved, &resp).ok()));

    eprintln!(
        "stubhouse app: async response {} (history: {})",
        resp.status,
        history_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "skipped".into())
    );
    Ok(ResponseDto::from_response(resp, history_id))
}

#[tauri::command]
fn open_workspace(path: String, state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let ws = Workspace::open(&path)
        .or_else(|e| match e {
            stubhouse_core::WorkspaceError::ManifestMissing(_) => {
                let default_name = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("workspace")
                    .to_string();
                Workspace::init(&path, &default_name)
            }
            other => Err(other),
        })
        .map_err(|e| e.to_string())?;

    let info = WorkspaceInfo {
        root: ws.root().to_path_buf(),
        manifest: ws.manifest().clone(),
    };
    let history = History::open(ws.root()).map_err(|e| e.to_string())?;

    *state.workspace.lock().unwrap() = Some(ws);
    *state.history.lock().unwrap() = Some(history);
    *state.active_env.lock().unwrap() = None;
    stop_mock_runtime(&state);
    Ok(info)
}

fn workspace_root(state: &AppState) -> Result<PathBuf, String> {
    let guard = state.workspace.lock().unwrap();
    guard
        .as_ref()
        .map(|ws| ws.root().to_path_buf())
        .ok_or_else(|| "no workspace open".to_string())
}

#[derive(Debug, Serialize)]
struct ActiveEnvironment {
    name: String,
    variables: std::collections::HashMap<String, String>,
}

#[tauri::command]
fn list_envs(state: State<'_, AppState>) -> Result<Vec<EnvironmentEntry>, String> {
    let root = workspace_root(&state)?;
    list_environments(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn activate_env(name: String, state: State<'_, AppState>) -> Result<ActiveEnvironment, String> {
    let root = workspace_root(&state)?;
    let env = load_environment(&root, &name).map_err(|e| e.to_string())?;
    let dto = ActiveEnvironment {
        name: env.name.clone(),
        variables: env.variables.clone(),
    };
    *state.active_env.lock().unwrap() = Some(env);
    Ok(dto)
}

#[tauri::command]
fn deactivate_env(state: State<'_, AppState>) -> Result<(), String> {
    *state.active_env.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
fn active_env(state: State<'_, AppState>) -> Result<Option<ActiveEnvironment>, String> {
    let guard = state.active_env.lock().unwrap();
    Ok(guard.as_ref().map(|env| ActiveEnvironment {
        name: env.name.clone(),
        variables: env.variables.clone(),
    }))
}

#[tauri::command]
fn save_env(env: EnvironmentFile, state: State<'_, AppState>) -> Result<(), String> {
    let root = workspace_root(&state)?;
    save_environment(&root, &env).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_mock_scenarios(state: State<'_, AppState>) -> Result<Vec<ScenarioEntry>, String> {
    let root = workspace_root(&state)?;
    list_scenarios(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn activate_mock_scenario(
    name: String,
    state: State<'_, AppState>,
) -> Result<ScenarioActivation, String> {
    let root = workspace_root(&state)?;
    activate_scenario(&root, &name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_mock_server(
    bind: Option<String>,
    port: Option<u16>,
    state: State<'_, AppState>,
) -> Result<MockServerStatus, String> {
    let root = workspace_root(&state)?;
    let bind = bind.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = port.unwrap_or(4000);
    let addr: std::net::SocketAddr = format!("{bind}:{port}")
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let rules = load_rules(&root).map_err(|e| e.to_string())?;

    // Fail fast for the common port-conflict case. The spawned server will bind
    // again immediately after this check.
    let probe = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("bind {addr}: {e}"))?;
    drop(probe);

    stop_mock_runtime(&state);

    let logs = Arc::new(Mutex::new(Vec::<MockLog>::new()));
    let log_sink = Arc::clone(&logs);
    let log_fn: Arc<dyn Fn(MockLog) + Send + Sync> = Arc::new(move |log| {
        let mut guard = log_sink.lock().unwrap();
        guard.push(log);
        if guard.len() > 200 {
            let overflow = guard.len() - 200;
            guard.drain(0..overflow);
        }
    });

    let rule_count = rules.len();
    let live_rule_count = Arc::new(AtomicUsize::new(rule_count));
    let reload_rule_count = Arc::clone(&live_rule_count);
    let reload_fn: Arc<dyn Fn(MockReload) + Send + Sync> = Arc::new(move |reload| {
        if reload.error.is_none() {
            reload_rule_count.store(reload.rules, Ordering::Relaxed);
        } else if let Some(error) = reload.error {
            eprintln!(
                "stubhouse mock reload failed; keeping {} rule(s): {error}",
                reload.rules
            );
        }
    });
    let (tx, rx) = tokio::sync::oneshot::channel();
    let root_for_server = root.clone();
    tokio::spawn(async move {
        if let Err(e) = run_with_hot_reload(
            root_for_server,
            addr,
            Some(rx),
            Some(log_fn),
            Some(reload_fn),
        )
        .await
        {
            eprintln!("stubhouse mock server stopped: {e}");
        }
    });

    let runtime = MockServerRuntime {
        bind,
        port,
        rules: live_rule_count,
        logs,
        shutdown: Some(tx),
    };
    let status = mock_status_from_runtime(&runtime);
    *state.mock_server.lock().unwrap() = Some(runtime);
    Ok(status)
}

#[tauri::command]
fn stop_mock_server(state: State<'_, AppState>) -> Result<MockServerStatus, String> {
    stop_mock_runtime(&state);
    Ok(MockServerStatus::stopped())
}

#[tauri::command]
fn mock_server_status(state: State<'_, AppState>) -> Result<MockServerStatus, String> {
    let guard = state.mock_server.lock().unwrap();
    Ok(guard
        .as_ref()
        .map(mock_status_from_runtime)
        .unwrap_or_else(MockServerStatus::stopped))
}

fn stop_mock_runtime(state: &AppState) {
    let mut guard = state.mock_server.lock().unwrap();
    if let Some(mut runtime) = guard.take() {
        if let Some(tx) = runtime.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

fn mock_status_from_runtime(runtime: &MockServerRuntime) -> MockServerStatus {
    let logs = runtime.logs.lock().unwrap().clone();
    MockServerStatus {
        running: true,
        bind: runtime.bind.clone(),
        port: runtime.port,
        url: format!("http://{}:{}", runtime.bind, runtime.port),
        rules: runtime.rules.load(Ordering::Relaxed),
        logs,
    }
}

#[tauri::command]
fn export_curl(req: Compose, state: State<'_, AppState>) -> Result<String, String> {
    let resolved = {
        let guard = state.active_env.lock().unwrap();
        match guard.as_ref() {
            Some(env) => interpolate_compose(&req, &env.variables),
            None => req,
        }
    };
    to_curl(&resolved).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
struct ImportSummary {
    imported: usize,
    collections: Vec<String>,
}

#[tauri::command]
fn import_postman(path: String, state: State<'_, AppState>) -> Result<ImportSummary, String> {
    let json = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let items = from_postman_v21(&json).map_err(|e| e.to_string())?;
    save_imported_items(items, state)
}

#[tauri::command]
fn import_insomnia(path: String, state: State<'_, AppState>) -> Result<ImportSummary, String> {
    let json = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let items = from_insomnia_v4(&json).map_err(|e| e.to_string())?;
    save_imported_items(items, state)
}

#[tauri::command]
fn import_har(path: String, state: State<'_, AppState>) -> Result<ImportSummary, String> {
    let json = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let items = from_har(&json).map_err(|e| e.to_string())?;
    save_imported_items(items, state)
}

#[tauri::command]
fn import_bruno(path: String, state: State<'_, AppState>) -> Result<ImportSummary, String> {
    let source = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let items = from_bruno_bru(&source).map_err(|e| e.to_string())?;
    save_imported_items(items, state)
}

fn save_imported_items(
    items: Vec<ImportedRequest>,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;

    let mut collections: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for item in &items {
        ws.save_request(&item.collection, &item.slug, &item.def)
            .map_err(|e| e.to_string())?;
        collections.insert(item.collection.clone());
    }
    Ok(ImportSummary {
        imported: items.len(),
        collections: collections.into_iter().collect(),
    })
}

#[tauri::command]
fn list_requests(state: State<'_, AppState>) -> Result<Vec<RequestEntry>, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    ws.list_requests().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_request(id: String, state: State<'_, AppState>) -> Result<RequestDefinition, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    ws.load_request(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_request(
    collection: String,
    slug: String,
    def: RequestDefinition,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    ws.save_request(&collection, &slug, &def)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_history(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HistoryEntry>, String> {
    let guard = state.history.lock().unwrap();
    let h = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    h.list(limit.unwrap_or(100)).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_history(id: i64, state: State<'_, AppState>) -> Result<HistoryReplay, String> {
    let guard = state.history.lock().unwrap();
    let h = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    let record = h.get(id).map_err(|e| e.to_string())?;
    let response = ResponseDto::from_response(record.response, Some(record.entry.id));
    Ok(HistoryReplay {
        request: record.request,
        response,
    })
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<usize, String> {
    let guard = state.history.lock().unwrap();
    let h = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;
    h.clear().map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_tests(state: State<'_, AppState>) -> Result<TestRunResult, String> {
    let root = workspace_root(&state)?;
    let env = state.active_env.lock().unwrap().clone();
    run_workspace_tests(&root, env.as_ref()).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_request,
            open_workspace,
            list_requests,
            load_request,
            save_request,
            list_history,
            load_history,
            clear_history,
            list_envs,
            activate_env,
            deactivate_env,
            active_env,
            save_env,
            start_send_request,
            poll_send_result,
            list_mock_scenarios,
            activate_mock_scenario,
            start_mock_server,
            stop_mock_server,
            mock_server_status,
            export_curl,
            import_postman,
            import_insomnia,
            import_har,
            import_bruno,
            run_tests,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
