#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use stubhouse_core::{
    send, Compose, RequestDefinition, RequestEntry, Response, Workspace, WorkspaceManifest,
};
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    workspace: Mutex<Option<Workspace>>,
}

#[derive(Debug, Serialize)]
struct ResponseDto {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
    elapsed_ms: u64,
    size_bytes: usize,
}

impl From<Response> for ResponseDto {
    fn from(r: Response) -> Self {
        Self {
            status: r.status,
            headers: r.headers,
            body: String::from_utf8_lossy(&r.body).into_owned(),
            elapsed_ms: r.elapsed_ms,
            size_bytes: r.size_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
struct WorkspaceInfo {
    root: PathBuf,
    manifest: WorkspaceManifest,
}

#[tauri::command]
async fn send_request(req: Compose) -> Result<ResponseDto, String> {
    let wire = req.build().map_err(|e| e.to_string())?;
    send(wire).await.map(ResponseDto::from).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_workspace(
    path: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, String> {
    let ws = Workspace::open(&path).or_else(|e| match e {
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

    let info = WorkspaceInfo { root: ws.root().to_path_buf(), manifest: ws.manifest().clone() };
    *state.workspace.lock().unwrap() = Some(ws);
    Ok(info)
}

#[tauri::command]
fn list_requests(state: State<'_, AppState>) -> Result<Vec<RequestEntry>, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard.as_ref().ok_or_else(|| "no workspace open".to_string())?;
    ws.list_requests().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_request(id: String, state: State<'_, AppState>) -> Result<RequestDefinition, String> {
    let guard = state.workspace.lock().unwrap();
    let ws = guard.as_ref().ok_or_else(|| "no workspace open".to_string())?;
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
    let ws = guard.as_ref().ok_or_else(|| "no workspace open".to_string())?;
    ws.save_request(&collection, &slug, &def).map_err(|e| e.to_string())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
