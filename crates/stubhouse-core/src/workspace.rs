use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::compose::Compose;

pub const MANIFEST_FILENAME: &str = "workspace.yaml";
pub const COLLECTIONS_DIR: &str = "collections";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceManifest {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String { "1".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(flatten)]
    pub compose: Compose,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestEntry {
    /// Path relative to workspace root, e.g. "collections/users/get-user.yaml".
    pub id: String,
    pub name: String,
    pub collection: String,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("workspace manifest not found at {0}")]
    ManifestMissing(PathBuf),
    #[error("request not found: {0}")]
    RequestNotFound(String),
    #[error("invalid request id (must stay within the workspace): {0}")]
    InvalidRequestId(String),
    #[error("invalid collection name: {0}")]
    InvalidCollectionName(String),
    #[error("invalid request slug: {0}")]
    InvalidRequestSlug(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug)]
pub struct Workspace {
    root: PathBuf,
    manifest: WorkspaceManifest,
}

impl Workspace {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let root = root.as_ref().to_path_buf();
        let manifest_path = root.join(MANIFEST_FILENAME);
        if !manifest_path.exists() {
            return Err(WorkspaceError::ManifestMissing(manifest_path));
        }
        let manifest: WorkspaceManifest =
            serde_yaml::from_str(&fs::read_to_string(&manifest_path)?)?;
        Ok(Self { root, manifest })
    }

    pub fn init(root: impl AsRef<Path>, name: &str) -> Result<Self, WorkspaceError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join(COLLECTIONS_DIR))?;
        let manifest = WorkspaceManifest { name: name.into(), version: default_version() };
        let manifest_path = root.join(MANIFEST_FILENAME);
        fs::write(&manifest_path, serde_yaml::to_string(&manifest)?)?;
        Ok(Self { root, manifest })
    }

    pub fn root(&self) -> &Path { &self.root }
    pub fn manifest(&self) -> &WorkspaceManifest { &self.manifest }

    pub fn list_requests(&self) -> Result<Vec<RequestEntry>, WorkspaceError> {
        let mut out = Vec::new();
        let collections_root = self.root.join(COLLECTIONS_DIR);
        if !collections_root.exists() {
            return Ok(out);
        }
        for col in sorted_dir_entries(&collections_root)? {
            if !col.file_type()?.is_dir() { continue; }
            let col_name = col.file_name().to_string_lossy().into_owned();
            let col_path = col.path();
            for req in sorted_dir_entries(&col_path)? {
                let path = req.path();
                if path.extension().and_then(|s| s.to_str()) != Some("yaml") { continue; }
                if !req.file_type()?.is_file() { continue; }

                let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();
                let id = rel.to_string_lossy().replace('\\', "/");

                let name = match read_request(&path) {
                    Ok(def) => def.name,
                    Err(_) => path.file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| id.clone()),
                };

                out.push(RequestEntry { id, name, collection: col_name.clone() });
            }
        }
        Ok(out)
    }

    pub fn load_request(&self, id: &str) -> Result<RequestDefinition, WorkspaceError> {
        let path = self.resolve_request_path(id)?;
        if !path.exists() {
            return Err(WorkspaceError::RequestNotFound(id.into()));
        }
        read_request(&path)
    }

    pub fn save_request(
        &self,
        collection: &str,
        slug: &str,
        def: &RequestDefinition,
    ) -> Result<String, WorkspaceError> {
        if !is_safe_name_segment(collection) {
            return Err(WorkspaceError::InvalidCollectionName(collection.into()));
        }
        if !is_safe_name_segment(slug) {
            return Err(WorkspaceError::InvalidRequestSlug(slug.into()));
        }
        let col_dir = self.root.join(COLLECTIONS_DIR).join(collection);
        fs::create_dir_all(&col_dir)?;
        let path = col_dir.join(format!("{slug}.yaml"));
        fs::write(&path, serde_yaml::to_string(def)?)?;
        let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();
        Ok(rel.to_string_lossy().replace('\\', "/"))
    }

    fn resolve_request_path(&self, id: &str) -> Result<PathBuf, WorkspaceError> {
        if id.contains("..") || id.starts_with('/') {
            return Err(WorkspaceError::InvalidRequestId(id.into()));
        }
        let path = self.root.join(id);
        let canonical_root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
        if !canonical_path.starts_with(&canonical_root) {
            return Err(WorkspaceError::InvalidRequestId(id.into()));
        }
        Ok(path)
    }
}

fn read_request(path: &Path) -> Result<RequestDefinition, WorkspaceError> {
    let s = fs::read_to_string(path)?;
    let def: RequestDefinition = serde_yaml::from_str(&s)?;
    Ok(def)
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<fs::DirEntry>, std::io::Error> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn is_safe_name_segment(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 120
        && !s.starts_with('.')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::{Auth, Body, Compose};
    use crate::http::Method;
    use tempfile::TempDir;

    fn sample_def(name: &str) -> RequestDefinition {
        RequestDefinition {
            name: name.into(),
            description: "example".into(),
            compose: Compose {
                method: Method::Get,
                url: "https://example.com/users".into(),
                query: vec![("page".into(), "1".into())],
                headers: vec![],
                auth: Auth::Bearer { token: "t".into() },
                body: Body::None,
            },
        }
    }

    #[test]
    fn manifest_round_trip() {
        let m = WorkspaceManifest { name: "demo".into(), version: "1".into() };
        let y = serde_yaml::to_string(&m).unwrap();
        let back: WorkspaceManifest = serde_yaml::from_str(&y).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn request_round_trip_with_auth_and_query() {
        let def = sample_def("Get Users");
        let y = serde_yaml::to_string(&def).unwrap();
        assert!(y.contains("kind: bearer"));
        let back: RequestDefinition = serde_yaml::from_str(&y).unwrap();
        assert_eq!(def, back);
    }

    #[test]
    fn init_creates_manifest_and_collections_dir() {
        let dir = TempDir::new().unwrap();
        let ws = Workspace::init(dir.path(), "demo").unwrap();
        assert_eq!(ws.manifest().name, "demo");
        assert!(dir.path().join("workspace.yaml").exists());
        assert!(dir.path().join("collections").is_dir());
    }

    #[test]
    fn open_errors_when_missing_manifest() {
        let dir = TempDir::new().unwrap();
        let err = Workspace::open(dir.path()).unwrap_err();
        matches!(err, WorkspaceError::ManifestMissing(_));
    }

    #[test]
    fn save_then_list_and_load() {
        let dir = TempDir::new().unwrap();
        let ws = Workspace::init(dir.path(), "demo").unwrap();

        let id = ws.save_request("users", "get-users", &sample_def("Get Users")).unwrap();
        assert_eq!(id, "collections/users/get-users.yaml");

        let entries = ws.list_requests().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id);
        assert_eq!(entries[0].name, "Get Users");
        assert_eq!(entries[0].collection, "users");

        let loaded = ws.load_request(&id).unwrap();
        assert_eq!(loaded.name, "Get Users");
        assert_eq!(loaded.compose.query, vec![("page".into(), "1".into())]);
    }

    #[test]
    fn save_rejects_path_traversal_in_collection() {
        let dir = TempDir::new().unwrap();
        let ws = Workspace::init(dir.path(), "demo").unwrap();
        let err = ws.save_request("../evil", "x", &sample_def("x")).unwrap_err();
        matches!(err, WorkspaceError::InvalidCollectionName(_));
    }

    #[test]
    fn load_rejects_path_traversal_in_id() {
        let dir = TempDir::new().unwrap();
        let ws = Workspace::init(dir.path(), "demo").unwrap();
        let err = ws.load_request("../../etc/passwd").unwrap_err();
        matches!(err, WorkspaceError::InvalidRequestId(_));
    }

    #[test]
    fn list_requests_ignores_non_yaml_files() {
        let dir = TempDir::new().unwrap();
        let ws = Workspace::init(dir.path(), "demo").unwrap();
        ws.save_request("users", "get-users", &sample_def("Get Users")).unwrap();

        let extra = dir.path().join("collections/users/notes.md");
        std::fs::write(&extra, "not a request").unwrap();

        let entries = ws.list_requests().unwrap();
        assert_eq!(entries.len(), 1);
    }
}
