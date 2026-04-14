use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use rusqlite::{params, Connection};
use serde::Serialize;
use thiserror::Error;

use crate::compose::Compose;
use crate::http::Response;

pub const HISTORY_DIR: &str = ".stubhouse";
pub const HISTORY_DB: &str = "history.db";

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    status INTEGER NOT NULL,
    elapsed_ms INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    request_json TEXT NOT NULL,
    response_headers_json TEXT NOT NULL,
    response_body BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_ts ON history(ts DESC);
"#;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub ts: i64,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryRecord {
    #[serde(flatten)]
    pub entry: HistoryEntry,
    pub request: Compose,
    pub response: Response,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("history entry not found: {0}")]
    NotFound(i64),
}

pub struct History {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl std::fmt::Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History").field("path", &self.path).finish()
    }
}

impl History {
    pub fn open(workspace_root: impl AsRef<Path>) -> Result<Self, HistoryError> {
        let dir = workspace_root.as_ref().join(HISTORY_DIR);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(HISTORY_DB);
        let conn = Connection::open(&path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { path, conn: Mutex::new(conn) })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record(&self, req: &Compose, resp: &Response) -> Result<i64, HistoryError> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let method = format!("{:?}", req.method).to_uppercase();
        let request_json = serde_json::to_string(req)?;
        let headers_json = serde_json::to_string(&resp.headers)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO history \
             (ts, method, url, status, elapsed_ms, size_bytes, request_json, response_headers_json, response_body) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                ts,
                method,
                req.url,
                resp.status as i64,
                resp.elapsed_ms as i64,
                resp.size_bytes as i64,
                request_json,
                headers_json,
                resp.body.as_ref(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<HistoryEntry>, HistoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, ts, method, url, status, elapsed_ms, size_bytes \
             FROM history ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok(HistoryEntry {
                id: r.get(0)?,
                ts: r.get(1)?,
                method: r.get(2)?,
                url: r.get(3)?,
                status: r.get::<_, i64>(4)? as u16,
                elapsed_ms: r.get::<_, i64>(5)? as u64,
                size_bytes: r.get::<_, i64>(6)? as usize,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get(&self, id: i64) -> Result<HistoryRecord, HistoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, ts, method, url, status, elapsed_ms, size_bytes, \
                    request_json, response_headers_json, response_body \
             FROM history WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        let row = rows.next()?.ok_or(HistoryError::NotFound(id))?;
        let entry = HistoryEntry {
            id: row.get(0)?,
            ts: row.get(1)?,
            method: row.get(2)?,
            url: row.get(3)?,
            status: row.get::<_, i64>(4)? as u16,
            elapsed_ms: row.get::<_, i64>(5)? as u64,
            size_bytes: row.get::<_, i64>(6)? as usize,
        };
        let request_json: String = row.get(7)?;
        let headers_json: String = row.get(8)?;
        let body_bytes: Vec<u8> = row.get(9)?;

        let request: Compose = serde_json::from_str(&request_json)?;
        let headers: Vec<(String, String)> = serde_json::from_str(&headers_json)?;
        let response = Response {
            status: entry.status,
            headers,
            body: Bytes::from(body_bytes),
            elapsed_ms: entry.elapsed_ms,
            size_bytes: entry.size_bytes,
        };
        Ok(HistoryRecord { entry, request, response })
    }

    pub fn clear(&self) -> Result<usize, HistoryError> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM history", [])?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::{Auth, Body, Compose};
    use crate::http::Method;
    use tempfile::TempDir;

    fn sample_compose() -> Compose {
        Compose {
            method: Method::Get,
            url: "https://example.com/users".into(),
            query: vec![("page".into(), "1".into())],
            headers: vec![("x-req".into(), "a".into())],
            auth: Auth::None,
            body: Body::None,
        }
    }

    fn sample_response(body: &[u8]) -> Response {
        Response {
            status: 200,
            headers: vec![("content-type".into(), "application/json".into())],
            body: Bytes::copy_from_slice(body),
            elapsed_ms: 42,
            size_bytes: body.len(),
        }
    }

    #[test]
    fn record_then_list_and_get_round_trips() {
        let dir = TempDir::new().unwrap();
        let h = History::open(dir.path()).unwrap();

        let id = h.record(&sample_compose(), &sample_response(b"{\"ok\":true}")).unwrap();
        assert!(id > 0);

        let entries = h.list(50).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id);
        assert_eq!(entries[0].method, "GET");
        assert_eq!(entries[0].url, "https://example.com/users");
        assert_eq!(entries[0].status, 200);
        assert_eq!(entries[0].size_bytes, 11);

        let full = h.get(id).unwrap();
        assert_eq!(full.request.url, "https://example.com/users");
        assert_eq!(full.request.query, vec![("page".into(), "1".into())]);
        assert_eq!(full.response.body.as_ref(), b"{\"ok\":true}");
        assert_eq!(full.response.headers.len(), 1);
    }

    #[test]
    fn list_returns_most_recent_first_and_respects_limit() {
        let dir = TempDir::new().unwrap();
        let h = History::open(dir.path()).unwrap();
        for i in 0..5 {
            let mut c = sample_compose();
            c.url = format!("https://example.com/{i}");
            h.record(&c, &sample_response(b"x")).unwrap();
        }
        let all = h.list(10).unwrap();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].url, "https://example.com/4");
        assert_eq!(all[4].url, "https://example.com/0");

        let top2 = h.list(2).unwrap();
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].url, "https://example.com/4");
    }

    #[test]
    fn get_returns_not_found_for_missing_id() {
        let dir = TempDir::new().unwrap();
        let h = History::open(dir.path()).unwrap();
        let err = h.get(999).unwrap_err();
        matches!(err, HistoryError::NotFound(_));
    }

    #[test]
    fn clear_empties_the_table() {
        let dir = TempDir::new().unwrap();
        let h = History::open(dir.path()).unwrap();
        h.record(&sample_compose(), &sample_response(b"a")).unwrap();
        h.record(&sample_compose(), &sample_response(b"b")).unwrap();
        let removed = h.clear().unwrap();
        assert_eq!(removed, 2);
        assert!(h.list(10).unwrap().is_empty());
    }

    #[test]
    fn open_creates_stubhouse_dir_and_db_file() {
        let dir = TempDir::new().unwrap();
        let h = History::open(dir.path()).unwrap();
        assert!(dir.path().join(".stubhouse").is_dir());
        assert!(h.path().exists());
    }
}
