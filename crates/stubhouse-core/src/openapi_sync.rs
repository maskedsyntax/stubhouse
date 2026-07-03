use std::collections::BTreeMap;

use serde::Deserialize;

use crate::history::HistoryRecord;
use crate::http::Method;

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct DriftIssue {
    pub history_id: i64,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub message: String,
}

pub fn detect_openapi_drift(
    source: &str,
    history: &[HistoryRecord],
) -> Result<Vec<DriftIssue>, serde_yaml::Error> {
    let spec: DriftOpenApiSpec = serde_yaml::from_str(source)?;
    let mut issues = Vec::new();
    for record in history {
        let method = method_name(record.request.method).to_ascii_lowercase();
        let path = url_path(&record.request.url);
        let Some(path_item) = spec.paths.get(&path) else {
            continue;
        };
        let Some(operation) = path_item.operations.get(&method) else {
            continue;
        };
        let Some(response) = operation
            .responses
            .get(&record.response.status.to_string())
            .or_else(|| operation.responses.get("default"))
        else {
            issues.push(issue(record, &path, "response status is not documented"));
            continue;
        };
        let Some(schema) = response
            .content
            .get("application/json")
            .and_then(|media| media.schema.as_ref())
        else {
            continue;
        };
        let Ok(json) = serde_json::from_slice::<serde_json::Value>(&record.response.body) else {
            issues.push(issue(record, &path, "response body is not valid JSON"));
            continue;
        };
        let mut messages = Vec::new();
        validate_schema(schema, &json, "$", &mut messages);
        for message in messages {
            issues.push(issue(record, &path, message));
        }
    }
    Ok(issues)
}

fn issue(record: &HistoryRecord, path: &str, message: impl Into<String>) -> DriftIssue {
    DriftIssue {
        history_id: record.entry.id,
        method: method_name(record.request.method).into(),
        path: path.into(),
        status: record.response.status,
        message: message.into(),
    }
}

fn validate_schema(
    schema: &OpenApiSchema,
    value: &serde_json::Value,
    path: &str,
    messages: &mut Vec<String>,
) {
    if let Some(kind) = schema.kind.as_deref() {
        let ok = match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            _ => true,
        };
        if !ok {
            messages.push(format!("{path} expected {kind}, got {}", json_kind(value)));
            return;
        }
    }

    if let Some(required) = &schema.required {
        if let Some(object) = value.as_object() {
            for key in required {
                if !object.contains_key(key) {
                    messages.push(format!("{path}.{key} is required but missing"));
                }
            }
        }
    }

    if let (Some(properties), Some(object)) = (&schema.properties, value.as_object()) {
        for (key, child_schema) in properties {
            if let Some(child) = object.get(key) {
                validate_schema(child_schema, child, &format!("{path}.{key}"), messages);
            }
        }
    }

    if let (Some(items), Some(values)) = (&schema.items, value.as_array()) {
        for (idx, item) in values.iter().enumerate() {
            validate_schema(items, item, &format!("{path}[{idx}]"), messages);
        }
    }
}

fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn url_path(url: &str) -> String {
    url::Url::parse(url)
        .map(|url| url.path().to_string())
        .unwrap_or_else(|_| url.split('?').next().unwrap_or(url).to_string())
}

fn method_name(method: Method) -> &'static str {
    match method {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
    }
}

#[derive(Deserialize)]
struct DriftOpenApiSpec {
    #[serde(default)]
    paths: BTreeMap<String, DriftPathItem>,
}

#[derive(Default, Deserialize)]
struct DriftPathItem {
    #[serde(flatten)]
    operations: BTreeMap<String, DriftOperation>,
}

#[derive(Default, Deserialize)]
struct DriftOperation {
    #[serde(default)]
    responses: BTreeMap<String, DriftResponse>,
}

#[derive(Default, Deserialize)]
struct DriftResponse {
    #[serde(default)]
    content: BTreeMap<String, DriftMediaType>,
}

#[derive(Default, Deserialize)]
struct DriftMediaType {
    #[serde(default)]
    schema: Option<OpenApiSchema>,
}

#[derive(Default, Deserialize)]
struct OpenApiSchema {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    required: Option<Vec<String>>,
    #[serde(default)]
    properties: Option<BTreeMap<String, OpenApiSchema>>,
    #[serde(default)]
    items: Option<Box<OpenApiSchema>>,
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;
    use crate::{
        compose::{Auth, Body, Compose},
        history::{HistoryEntry, HistoryRecord},
        http::Response,
    };

    #[test]
    fn detects_missing_required_response_field() {
        let spec = r#"
openapi: 3.0.3
paths:
  /users:
    get:
      responses:
        "200":
          content:
            application/json:
              schema:
                type: object
                required: [id]
                properties:
                  id:
                    type: string
"#;
        let record = record(Bytes::from_static(br#"{"name":"Alice"}"#));
        let issues = detect_openapi_drift(spec, &[record]).unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("required"));
    }

    fn record(body: Bytes) -> HistoryRecord {
        HistoryRecord {
            entry: HistoryEntry {
                id: 7,
                ts: 0,
                method: "GET".into(),
                url: "https://api.example.com/users".into(),
                status: 200,
                elapsed_ms: 10,
                size_bytes: body.len(),
            },
            request: Compose {
                method: Method::Get,
                url: "https://api.example.com/users".into(),
                query: vec![],
                headers: vec![],
                auth: Auth::None,
                body: Body::None,
            },
            response: Response {
                status: 200,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body,
                elapsed_ms: 10,
                size_bytes: 16,
            },
        }
    }
}
