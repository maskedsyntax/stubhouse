#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use stubhouse_core::{
    from_postman_v21, interpolate_compose, list_environments, load_environment, save_environment,
    send, to_curl, Compose, Environment, EnvironmentEntry, EnvironmentFile, History, HistoryEntry,
    RequestDefinition, RequestEntry, Response, Workspace, WorkspaceManifest,
};
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    workspace: Mutex<Option<Workspace>>,
    history: Mutex<Option<History>>,
    active_env: Mutex<Option<Environment>>,
}

#[derive(Debug, Serialize)]
struct ResponseDto {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
    elapsed_ms: u64,
    size_bytes: usize,
    history_id: Option<i64>,
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

#[tauri::command]
async fn send_request(req: Compose, state: State<'_, AppState>) -> Result<ResponseDto, String> {
    let resolved = {
        let guard = state.active_env.lock().unwrap();
        match guard.as_ref() {
            Some(env) => interpolate_compose(&req, &env.variables),
            None => req.clone(),
        }
    };
    let wire = resolved.clone().build().map_err(|e| e.to_string())?;
    let resp = send(wire).await.map_err(|e| e.to_string())?;

    let history_id = {
        let guard = state.history.lock().unwrap();
        guard.as_ref().and_then(|h| h.record(&resolved, &resp).ok())
    };

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
    let guard = state.workspace.lock().unwrap();
    let ws = guard
        .as_ref()
        .ok_or_else(|| "no workspace open".to_string())?;

    let json = std::fs::read_to_string(&path).map_err(|e| format!("read {path}: {e}"))?;
    let items = from_postman_v21(&json).map_err(|e| e.to_string())?;

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
            export_curl,
            import_postman,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
