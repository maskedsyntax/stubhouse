//! Mock server runtime — rule schema, route matcher, and embedded hyper server.
//!
//! Phase 2 mock runtime:
//!   * YAML mock rules under `collections/*/mocks/*.yaml`
//!   * named response scenarios with one active scenario per rule
//!   * exact, `:param`, `*`, and `**` path matching
//!   * static body rendering with `{{params.NAME}}` interpolation

pub mod matcher;
pub mod server;

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::http::Method;

pub const MOCKS_SUBDIR: &str = "mocks";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockRule {
    pub name: String,
    pub method: Method,
    pub path: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub response: MockResponse,
    #[serde(default)]
    pub scenarios: Vec<MockScenario>,
    #[serde(default)]
    pub fault: Option<MockFault>,
    #[serde(default)]
    pub passthrough: bool,
    #[serde(default)]
    pub upstream_url: Option<String>,
    #[serde(default)]
    pub record: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingConfig {
    #[serde(default = "default_recordings_dir")]
    pub dir: PathBuf,
    #[serde(default)]
    pub scrub: ScrubConfig,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            dir: default_recordings_dir(),
            scrub: ScrubConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScrubConfig {
    #[serde(default = "default_scrub_replacement")]
    pub replacement: String,
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub json_fields: Vec<String>,
    #[serde(default)]
    pub text: Vec<String>,
}

impl Default for ScrubConfig {
    fn default() -> Self {
        Self {
            replacement: default_scrub_replacement(),
            headers: vec![],
            json_fields: vec![],
            text: vec![],
        }
    }
}

fn default_recordings_dir() -> PathBuf {
    PathBuf::from("recordings")
}

fn default_scrub_replacement() -> String {
    "[REDACTED]".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockResource {
    pub path: String,
    #[serde(default = "default_resource_id_field")]
    pub id_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_file: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub auto_crud: bool,
}

fn default_resource_id_field() -> String {
    "id".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockScenario {
    pub name: String,
    #[serde(default)]
    pub active: bool,
    pub response: MockResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioEntry {
    pub name: String,
    pub rules: usize,
    pub active_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioActivation {
    pub scenario: String,
    pub files_changed: usize,
    pub rules_changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: MockBody,
    /// Static delay in milliseconds before responding.
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MockFault {
    Kind(MockFaultKind),
    Config(MockFaultConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MockFaultKind {
    Timeout,
    SlowResponse,
    ConnectionReset,
    PartialBody,
    Random5xx,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockFaultConfig {
    pub kind: MockFaultKind,
    #[serde(default)]
    pub delay_ms: Option<u64>,
    #[serde(default)]
    pub probability: Option<f64>,
}

impl MockFault {
    pub fn kind(&self) -> MockFaultKind {
        match self {
            MockFault::Kind(kind) => kind.clone(),
            MockFault::Config(config) => config.kind.clone(),
        }
    }

    pub fn delay_ms(&self) -> u64 {
        match self {
            MockFault::Kind(MockFaultKind::SlowResponse) => 1_000,
            MockFault::Config(config) if config.kind == MockFaultKind::SlowResponse => {
                config.delay_ms.unwrap_or(1_000)
            }
            _ => 0,
        }
    }

    pub fn probability(&self) -> f64 {
        match self {
            MockFault::Kind(MockFaultKind::Random5xx) => 1.0,
            MockFault::Config(config) if config.kind == MockFaultKind::Random5xx => {
                config.probability.unwrap_or(1.0).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            status: 200,
            headers: vec![],
            body: MockBody::None,
            delay_ms: 0,
            body_script: None,
        }
    }
}

impl MockRule {
    pub fn active_response(&self) -> &MockResponse {
        self.scenarios
            .iter()
            .find(|scenario| scenario.active)
            .map(|scenario| &scenario.response)
            .unwrap_or(&self.response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MockBody {
    None,
    Text { content_type: String, text: String },
    Json { text: String },
}

impl Default for MockBody {
    fn default() -> Self {
        MockBody::None
    }
}

#[derive(Debug, Error)]
pub enum MockError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error in {file}: {source}")]
    Yaml {
        file: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("fixture {file} must contain a YAML sequence")]
    InvalidFixture { file: PathBuf },
    #[error("server error: {0}")]
    Server(String),
}

/// Recursively load every `collections/*/mocks/*.yaml` rule under `workspace_root`.
pub fn load_rules(workspace_root: &Path) -> Result<Vec<MockRule>, MockError> {
    let mut rules = load_rule_files(workspace_root)?
        .into_iter()
        .map(|(_, rule)| rule)
        .collect::<Vec<_>>();
    // Higher priority first. Stable ordering by file means ties are broken predictably.
    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    Ok(rules)
}

pub fn load_resources(
    workspace_root: &Path,
) -> Result<Vec<(MockResource, Vec<serde_json::Value>)>, MockError> {
    #[derive(Deserialize)]
    struct ResourceManifest {
        #[serde(default)]
        mock_resources: Vec<MockResource>,
    }

    let manifest_path = workspace_root.join(crate::workspace::MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Ok(vec![]);
    }
    let manifest: ResourceManifest = serde_yaml::from_str(&fs::read_to_string(&manifest_path)?)
        .map_err(|source| MockError::Yaml {
            file: manifest_path.clone(),
            source,
        })?;

    manifest
        .mock_resources
        .into_iter()
        .map(|resource| {
            let seed = match &resource.seed_file {
                Some(seed_file) => {
                    let seed_path = resolve_workspace_path(workspace_root, seed_file);
                    let value: serde_json::Value = serde_yaml::from_str(&fs::read_to_string(
                        &seed_path,
                    )?)
                    .map_err(|source| MockError::Yaml {
                        file: seed_path.clone(),
                        source,
                    })?;
                    value
                        .as_array()
                        .cloned()
                        .ok_or_else(|| MockError::InvalidFixture { file: seed_path })?
                }
                None => vec![],
            };
            Ok((resource, seed))
        })
        .collect()
}

pub fn load_recording_config(workspace_root: &Path) -> Result<RecordingConfig, MockError> {
    #[derive(Deserialize)]
    struct RecordingManifest {
        #[serde(default)]
        recording: RecordingConfig,
    }

    let manifest_path = workspace_root.join(crate::workspace::MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Ok(RecordingConfig::default());
    }
    let manifest: RecordingManifest = serde_yaml::from_str(&fs::read_to_string(&manifest_path)?)
        .map_err(|source| MockError::Yaml {
            file: manifest_path,
            source,
        })?;
    Ok(manifest.recording)
}

fn resolve_workspace_path(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

pub fn list_scenarios(workspace_root: &Path) -> Result<Vec<ScenarioEntry>, MockError> {
    let mut scenarios: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (_, rule) in load_rule_files(workspace_root)? {
        for scenario in &rule.scenarios {
            let entry = scenarios.entry(scenario.name.clone()).or_default();
            entry.0 += 1;
            if scenario.active {
                entry.1 += 1;
            }
        }
    }

    Ok(scenarios
        .into_iter()
        .map(|(name, (rules, active_rules))| ScenarioEntry {
            name,
            rules,
            active_rules,
        })
        .collect())
}

pub fn activate_scenario(
    workspace_root: &Path,
    scenario_name: &str,
) -> Result<ScenarioActivation, MockError> {
    let mut files_changed = 0usize;
    let mut rules_changed = 0usize;

    for (path, mut rule) in load_rule_files(workspace_root)? {
        if !rule
            .scenarios
            .iter()
            .any(|scenario| scenario.name == scenario_name)
        {
            continue;
        }

        let before = rule.scenarios.clone();
        for scenario in &mut rule.scenarios {
            scenario.active = scenario.name == scenario_name;
        }

        if rule.scenarios != before {
            let yaml = serde_yaml::to_string(&rule).map_err(|source| MockError::Yaml {
                file: path.clone(),
                source,
            })?;
            fs::write(&path, yaml)?;
            files_changed += 1;
            rules_changed += 1;
        }
    }

    Ok(ScenarioActivation {
        scenario: scenario_name.to_string(),
        files_changed,
        rules_changed,
    })
}

fn load_rule_files(workspace_root: &Path) -> Result<Vec<(PathBuf, MockRule)>, MockError> {
    let collections = workspace_root.join(crate::workspace::COLLECTIONS_DIR);
    if !collections.exists() {
        return Ok(vec![]);
    }
    let mut rules = Vec::new();
    for col in sorted_dir(&collections)? {
        if !col.file_type()?.is_dir() {
            continue;
        }
        let mocks = col.path().join(MOCKS_SUBDIR);
        if !mocks.exists() {
            continue;
        }
        for entry in sorted_dir(&mocks)? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            if !entry.file_type()?.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path)?;
            let rule: MockRule = serde_yaml::from_str(&text).map_err(|source| MockError::Yaml {
                file: path.clone(),
                source,
            })?;
            rules.push((path, rule));
        }
    }
    Ok(rules)
}

fn sorted_dir(dir: &Path) -> std::io::Result<Vec<fs::DirEntry>> {
    let mut v: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    v.sort_by_key(|e| e.file_name());
    Ok(v)
}

/// Interpolate `{{params.NAME}}` in a response body string against captured path params.
pub fn render_body(template: &str, params: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let key = after[..end].trim();
                if let Some(param_name) = key.strip_prefix("params.") {
                    match params.get(param_name) {
                        Some(v) => out.push_str(v),
                        None => out.push_str(&rest[start..start + 2 + end + 2]),
                    }
                } else {
                    out.push_str(&rest[start..start + 2 + end + 2]);
                }
                rest = &after[end + 2..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(p: &Path, s: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, s).unwrap();
    }

    #[test]
    fn rule_yaml_roundtrip() {
        let rule = MockRule {
            name: "Get user".into(),
            method: Method::Get,
            path: "/users/:id".into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: MockBody::Json {
                    text: r#"{"id":"{{params.id}}"}"#.into(),
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
        };
        let y = serde_yaml::to_string(&rule).unwrap();
        let back: MockRule = serde_yaml::from_str(&y).unwrap();
        assert_eq!(rule, back);
    }

    #[test]
    fn load_rules_finds_yaml_under_mocks_subdirs() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        write(
            &root.join("collections/users/mocks/get.yaml"),
            r#"
name: get-user
method: GET
path: /users/:id
response:
  status: 200
  body:
    kind: json
    text: '{"id":"{{params.id}}"}'
"#,
        );
        write(
            &root.join("collections/payments/mocks/charge.yaml"),
            r#"
name: charge
method: POST
path: /charges
priority: 10
response:
  status: 201
  body:
    kind: json
    text: '{"ok":true}'
"#,
        );
        let rules = load_rules(root).unwrap();
        assert_eq!(rules.len(), 2);
        // priority desc: charge (10) before get-user (0)
        assert_eq!(rules[0].name, "charge");
        assert_eq!(rules[1].name, "get-user");
    }

    #[test]
    fn load_rule_with_active_scenario() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        write(
            &root.join("collections/users/mocks/get.yaml"),
            r#"
name: get-user
method: GET
path: /users/:id
response:
  status: 200
  body:
    kind: json
    text: '{"id":"{{params.id}}","state":"default"}'
scenarios:
  - name: empty
    response:
      status: 404
      body:
        kind: json
        text: '{"error":"missing"}'
  - name: subscribed
    active: true
    response:
      status: 200
      body:
        kind: json
        text: '{"id":"{{params.id}}","plan":"pro"}'
"#,
        );

        let rules = load_rules(root).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].scenarios.len(), 2);
        assert_eq!(rules[0].active_response().status, 200);
        match &rules[0].active_response().body {
            MockBody::Json { text } => assert!(text.contains("\"plan\":\"pro\"")),
            other => panic!("expected json body, got {other:?}"),
        }
    }

    #[test]
    fn list_scenarios_counts_rules_and_active_rules() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        write(
            &root.join("collections/users/mocks/get.yaml"),
            r#"
name: get-user
method: GET
path: /users/:id
scenarios:
  - name: empty
    active: true
    response:
      status: 404
  - name: subscribed
    response:
      status: 200
"#,
        );
        write(
            &root.join("collections/posts/mocks/get.yaml"),
            r#"
name: get-post
method: GET
path: /posts/:id
scenarios:
  - name: subscribed
    active: true
    response:
      status: 200
"#,
        );

        let scenarios = list_scenarios(root).unwrap();
        assert_eq!(
            scenarios,
            vec![
                ScenarioEntry {
                    name: "empty".into(),
                    rules: 1,
                    active_rules: 1,
                },
                ScenarioEntry {
                    name: "subscribed".into(),
                    rules: 2,
                    active_rules: 1,
                },
            ]
        );
    }

    #[test]
    fn activate_scenario_updates_matching_rule_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let file = root.join("collections/users/mocks/get.yaml");
        write(
            &file,
            r#"
name: get-user
method: GET
path: /users/:id
scenarios:
  - name: empty
    active: true
    response:
      status: 404
  - name: subscribed
    response:
      status: 200
"#,
        );

        let activation = activate_scenario(root, "subscribed").unwrap();
        assert_eq!(activation.files_changed, 1);
        assert_eq!(activation.rules_changed, 1);

        let rule: MockRule = serde_yaml::from_str(&fs::read_to_string(file).unwrap()).unwrap();
        assert!(!rule.scenarios[0].active);
        assert!(rule.scenarios[1].active);
    }

    #[test]
    fn load_rules_skips_non_yaml() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        write(&root.join("collections/u/mocks/notes.md"), "hello");
        write(
            &root.join("collections/u/mocks/r.yaml"),
            "name: r\nmethod: GET\npath: /x\nresponse:\n  status: 200\n",
        );
        let rules = load_rules(root).unwrap();
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn load_rules_empty_when_no_collections() {
        let dir = TempDir::new().unwrap();
        let rules = load_rules(dir.path()).unwrap();
        assert!(rules.is_empty());
    }

    #[test]
    fn render_body_substitutes_params() {
        let mut params = HashMap::new();
        params.insert("id".into(), "42".into());
        let out = render_body(r#"{"id":"{{params.id}}"}"#, &params);
        assert_eq!(out, r#"{"id":"42"}"#);
    }

    #[test]
    fn render_body_keeps_unknown_placeholders() {
        let out = render_body("{{params.missing}}", &HashMap::new());
        assert_eq!(out, "{{params.missing}}");
        // Non-`params.` placeholders are also kept verbatim — they're not our job.
        let out = render_body("hello {{$timestamp}}", &HashMap::new());
        assert_eq!(out, "hello {{$timestamp}}");
    }
}
