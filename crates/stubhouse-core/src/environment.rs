use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ENVIRONMENTS_DIR: &str = "environments";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum SecretSource {
    Env {
        #[serde(default)]
        var: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentFile {
    pub name: String,
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub secrets: HashMap<String, SecretSource>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentEntry {
    pub name: String,
    pub file: String,
}

#[derive(Debug, Error)]
pub enum EnvironmentError {
    #[error("environment not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

fn env_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(ENVIRONMENTS_DIR)
}

pub fn list_environments(workspace_root: &Path) -> Result<Vec<EnvironmentEntry>, EnvironmentError> {
    let dir = env_dir(workspace_root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(&dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_name.clone());
        out.push(EnvironmentEntry {
            name,
            file: file_name,
        });
    }
    Ok(out)
}

pub fn load_environment(
    workspace_root: &Path,
    name: &str,
) -> Result<Environment, EnvironmentError> {
    let path = env_dir(workspace_root).join(format!("{name}.yaml"));
    if !path.exists() {
        return Err(EnvironmentError::NotFound(name.into()));
    }
    let content = fs::read_to_string(&path)?;
    let file: EnvironmentFile = serde_yaml::from_str(&content)?;
    Ok(resolve_environment(&file))
}

pub fn save_environment(
    workspace_root: &Path,
    env_file: &EnvironmentFile,
) -> Result<(), EnvironmentError> {
    let dir = env_dir(workspace_root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.yaml", env_file.name));
    fs::write(&path, serde_yaml::to_string(env_file)?)?;
    Ok(())
}

fn resolve_environment(file: &EnvironmentFile) -> Environment {
    let mut variables = HashMap::new();
    for (k, v) in &file.variables {
        let resolved = match v {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Null => String::new(),
            other => serde_yaml::to_string(other).unwrap_or_default(),
        };
        variables.insert(k.clone(), resolved);
    }
    for (k, source) in &file.secrets {
        match source {
            SecretSource::Env { var } => {
                let env_var = var.as_deref().unwrap_or(k.as_str());
                if let Ok(val) = std::env::var(env_var) {
                    variables.insert(k.clone(), val);
                }
            }
        }
    }
    Environment {
        name: file.name.clone(),
        variables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_env_file(name: &str) -> EnvironmentFile {
        let mut variables = HashMap::new();
        variables.insert(
            "base_url".into(),
            serde_yaml::Value::String("http://localhost:3000".into()),
        );
        variables.insert("timeout_ms".into(), serde_yaml::Value::Number(5000.into()));
        EnvironmentFile {
            name: name.into(),
            variables,
            secrets: HashMap::new(),
        }
    }

    #[test]
    fn save_and_list_environments() {
        let dir = TempDir::new().unwrap();
        save_environment(dir.path(), &make_env_file("dev")).unwrap();
        save_environment(dir.path(), &make_env_file("prod")).unwrap();

        let envs = list_environments(dir.path()).unwrap();
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0].name, "dev");
        assert_eq!(envs[1].name, "prod");
    }

    #[test]
    fn load_resolves_variables() {
        let dir = TempDir::new().unwrap();
        save_environment(dir.path(), &make_env_file("dev")).unwrap();

        let env = load_environment(dir.path(), "dev").unwrap();
        assert_eq!(env.name, "dev");
        assert_eq!(env.variables["base_url"], "http://localhost:3000");
        assert_eq!(env.variables["timeout_ms"], "5000");
    }

    #[test]
    fn load_resolves_secrets_from_os_env() {
        let dir = TempDir::new().unwrap();
        let mut ef = make_env_file("test");
        ef.secrets.insert(
            "api_key".into(),
            SecretSource::Env {
                var: Some("STUBHOUSE_TEST_SECRET_XYZ".into()),
            },
        );
        save_environment(dir.path(), &ef).unwrap();

        std::env::set_var("STUBHOUSE_TEST_SECRET_XYZ", "s3cret");
        let env = load_environment(dir.path(), "test").unwrap();
        assert_eq!(env.variables["api_key"], "s3cret");
        std::env::remove_var("STUBHOUSE_TEST_SECRET_XYZ");
    }

    #[test]
    fn load_not_found() {
        let dir = TempDir::new().unwrap();
        let err = load_environment(dir.path(), "nope").unwrap_err();
        assert!(matches!(err, EnvironmentError::NotFound(_)));
    }

    #[test]
    fn list_empty_when_no_dir() {
        let dir = TempDir::new().unwrap();
        let envs = list_environments(dir.path()).unwrap();
        assert!(envs.is_empty());
    }
}
