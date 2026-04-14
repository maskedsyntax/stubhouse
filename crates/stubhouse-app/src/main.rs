#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use stubhouse_core::{send, Method, Request, Response};

#[derive(Debug, Deserialize)]
struct RequestDto {
    method: Method,
    url: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    body: Option<String>,
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
        let body = String::from_utf8_lossy(&r.body).into_owned();
        Self {
            status: r.status,
            headers: r.headers,
            body,
            elapsed_ms: r.elapsed_ms,
            size_bytes: r.size_bytes,
        }
    }
}

#[tauri::command]
async fn send_request(req: RequestDto) -> Result<ResponseDto, String> {
    let core_req = Request {
        method: req.method,
        url: req.url,
        headers: req.headers,
        body: req.body.map(|s| Bytes::from(s.into_bytes())),
    };
    send(core_req).await.map(ResponseDto::from).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![send_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
