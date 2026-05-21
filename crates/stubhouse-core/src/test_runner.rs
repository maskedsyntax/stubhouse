//! Collection test runner for request definitions.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::{
    http::send, interpolate_compose, script::ScriptRuntime, Environment, RequestDefinition,
    Workspace,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TestAssertionResult {
    pub request_id: String,
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TestRunResult {
    pub assertions: Vec<TestAssertionResult>,
}

impl TestRunResult {
    pub fn total(&self) -> usize {
        self.assertions.len()
    }

    pub fn failed(&self) -> usize {
        self.assertions
            .iter()
            .filter(|result| !result.passed)
            .count()
    }

    pub fn passed(&self) -> usize {
        self.total() - self.failed()
    }

    pub fn success(&self) -> bool {
        self.failed() == 0
    }
}

pub async fn run_workspace_tests(
    workspace_root: &Path,
    env: Option<&Environment>,
) -> Result<TestRunResult, String> {
    let workspace = Workspace::open(workspace_root).map_err(|e| e.to_string())?;
    let entries = workspace.list_requests().map_err(|e| e.to_string())?;
    let mut variables = env
        .map(|env| env.variables.clone())
        .unwrap_or_else(HashMap::new);
    let mut assertions = Vec::new();

    for entry in entries {
        let def = workspace
            .load_request(&entry.id)
            .map_err(|e| e.to_string())?;
        let Some(script) = def.post_response_script.as_deref() else {
            continue;
        };
        let tests = extract_tests(script);
        if tests.is_empty() {
            continue;
        }

        let response = send_request_for_test(&def, &variables)
            .await
            .map_err(|e| format!("{}: {e}", entry.id))?;

        for test in tests {
            let runtime = ScriptRuntime::new();
            match runtime.eval_response_bool(&test.expr, &response, &Default::default()) {
                Ok(true) => assertions.push(TestAssertionResult {
                    request_id: entry.id.clone(),
                    name: test.name,
                    passed: true,
                    message: None,
                }),
                Ok(false) => assertions.push(TestAssertionResult {
                    request_id: entry.id.clone(),
                    name: test.name,
                    passed: false,
                    message: Some("expression returned false".into()),
                }),
                Err(e) => assertions.push(TestAssertionResult {
                    request_id: entry.id.clone(),
                    name: test.name,
                    passed: false,
                    message: Some(e.to_string()),
                }),
            }
        }

        let runtime = ScriptRuntime::new();
        if let Ok(outcome) = runtime.run_post_response(script, &response, &Default::default()) {
            variables.extend(outcome.variables);
        }
    }

    Ok(TestRunResult { assertions })
}

async fn send_request_for_test(
    def: &RequestDefinition,
    variables: &HashMap<String, String>,
) -> Result<crate::Response, String> {
    let mut compose = interpolate_compose(&def.compose, variables);
    if let Some(script) = def.pre_request_script.as_deref() {
        let runtime = ScriptRuntime::new();
        compose = runtime
            .run_pre_request(
                script,
                &compose,
                &crate::ScriptContext {
                    env: variables.clone(),
                    variables: variables.clone(),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    let wire = compose.build().map_err(|e| e.to_string())?;
    send(wire).await.map_err(|e| e.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestBlock {
    name: String,
    expr: String,
}

fn extract_tests(script: &str) -> Vec<TestBlock> {
    let mut tests = Vec::new();
    let mut rest = script;
    while let Some(start) = rest.find("test(\"") {
        let after = &rest[start + 6..];
        let Some(name_end) = after.find("\")") else {
            break;
        };
        let name = after[..name_end].to_string();
        let after_name = &after[name_end + 2..];
        let Some(open_brace) = after_name.find('{') else {
            break;
        };
        let after_open = &after_name[open_brace + 1..];
        let Some(close_brace) = after_open.find('}') else {
            break;
        };
        let expr = after_open[..close_brace].trim().to_string();
        tests.push(TestBlock { name, expr });
        rest = &after_open[close_brace + 1..];
    }
    tests
}

pub fn junit_xml(result: &TestRunResult) -> String {
    let mut xml = format!(
        r#"<testsuite name="stubhouse" tests="{}" failures="{}">"#,
        result.total(),
        result.failed()
    );
    for assertion in &result.assertions {
        xml.push_str(&format!(
            r#"<testcase classname="{}" name="{}">"#,
            escape_xml(&assertion.request_id),
            escape_xml(&assertion.name)
        ));
        if !assertion.passed {
            xml.push_str(&format!(
                r#"<failure message="{}"/>"#,
                escape_xml(assertion.message.as_deref().unwrap_or("failed"))
            ));
        }
        xml.push_str("</testcase>");
    }
    xml.push_str("</testsuite>");
    xml
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_test_blocks() {
        let tests = extract_tests(
            r#"
            test("Status is 200") { response.status == 200 }
            test("Fast enough") { response.elapsed_ms < 500 }
            "#,
        );
        assert_eq!(tests.len(), 2);
        assert_eq!(tests[0].name, "Status is 200");
        assert_eq!(tests[1].expr, "response.elapsed_ms < 500");
    }

    #[test]
    fn junit_xml_marks_failures() {
        let xml = junit_xml(&TestRunResult {
            assertions: vec![TestAssertionResult {
                request_id: "collections/a.yaml".into(),
                name: "A < B".into(),
                passed: false,
                message: Some("bad & worse".into()),
            }],
        });
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("A &lt; B"));
        assert!(xml.contains("bad &amp; worse"));
    }
}
