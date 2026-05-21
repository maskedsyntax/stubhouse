//! Importers for foreign collection formats.
//!
//! v1 supports Postman Collection v2.1. The mapping is best-effort:
//! - folders become collection names (one level deep — nested folders are flattened
//!   by joining with `-`)
//! - request scripts are dropped (description is preserved, with a marker if scripts
//!   existed in the source)
//! - auth maps for `bearer`, `basic`, `apikey`, `noauth`; other types fall back to None

use serde::Deserialize;
use thiserror::Error;

use crate::compose::{ApiKeyLocation, Auth, Body, Compose};
use crate::http::Method;
use crate::workspace::RequestDefinition;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported Postman schema: {0}")]
    Schema(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedRequest {
    pub collection: String,
    pub slug: String,
    pub def: RequestDefinition,
}

pub fn from_postman_v21(json: &str) -> Result<Vec<ImportedRequest>, ImportError> {
    let collection: PostmanCollection = serde_json::from_str(json)?;
    let mut out = Vec::new();
    walk_items(&collection.item, "imported", &mut out);
    Ok(out)
}

fn walk_items(items: &[PostmanItem], parent_collection: &str, out: &mut Vec<ImportedRequest>) {
    for item in items {
        match item {
            PostmanItem::Folder { name, item } => {
                let child = combine_collection(parent_collection, name);
                walk_items(item, &child, out);
            }
            PostmanItem::Request {
                name,
                request,
                event,
            } => {
                if let Some(def) = build_definition(name, request, event.as_deref()) {
                    let slug = slugify(name);
                    out.push(ImportedRequest {
                        collection: parent_collection.to_string(),
                        slug,
                        def,
                    });
                }
            }
        }
    }
}

fn combine_collection(parent: &str, name: &str) -> String {
    let s = sanitize_path_segment(name);
    if parent == "imported" {
        s
    } else {
        format!("{parent}-{s}")
    }
}

fn build_definition(
    name: &str,
    request: &PostmanRequest,
    events: Option<&[PostmanEvent]>,
) -> Option<RequestDefinition> {
    let method = parse_method(request.method.as_deref().unwrap_or("GET"));
    let url = match &request.url {
        Some(PostmanUrl::Raw(s)) => s.clone(),
        Some(PostmanUrl::Object {
            raw, host, path, ..
        }) => raw
            .clone()
            .or_else(|| {
                let host = host.as_ref()?.join(".");
                let path = path.as_ref().map(|p| p.join("/")).unwrap_or_default();
                Some(if path.is_empty() {
                    host
                } else {
                    format!("{host}/{path}")
                })
            })
            .unwrap_or_default(),
        None => String::new(),
    };

    let query = request
        .url
        .as_ref()
        .and_then(|u| match u {
            PostmanUrl::Object { query, .. } => query.as_ref(),
            _ => None,
        })
        .map(|q| {
            q.iter()
                .filter(|p| !p.disabled.unwrap_or(false))
                .map(|p| {
                    (
                        p.key.clone().unwrap_or_default(),
                        p.value.clone().unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let headers = request
        .header
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|h| !h.disabled.unwrap_or(false))
        .map(|h| (h.key.clone(), h.value.clone()))
        .collect();

    let auth = request.auth.as_ref().map(map_auth).unwrap_or(Auth::None);
    let body = request.body.as_ref().map(map_body).unwrap_or(Body::None);

    let has_scripts = events
        .map(|es| es.iter().any(|e| !e.script.exec.is_empty()))
        .unwrap_or(false);

    let mut description = request.description.clone().unwrap_or_default();
    if has_scripts {
        if !description.is_empty() {
            description.push_str("\n\n");
        }
        description.push_str("(Postman scripts were dropped during import.)");
    }

    Some(RequestDefinition {
        name: name.to_string(),
        description,
        pre_request_script: None,
        post_response_script: None,
        compose: Compose {
            method,
            url,
            query,
            headers,
            auth,
            body,
        },
    })
}

fn map_auth(a: &PostmanAuth) -> Auth {
    match a.kind.as_str() {
        "bearer" => Auth::Bearer {
            token: get_kv(&a.bearer, "token").unwrap_or_default(),
        },
        "basic" => Auth::Basic {
            username: get_kv(&a.basic, "username").unwrap_or_default(),
            password: get_kv(&a.basic, "password").unwrap_or_default(),
        },
        "apikey" => {
            let in_q = get_kv(&a.apikey, "in").as_deref() == Some("query");
            Auth::ApiKey {
                location: if in_q {
                    ApiKeyLocation::Query
                } else {
                    ApiKeyLocation::Header
                },
                name: get_kv(&a.apikey, "key").unwrap_or_default(),
                value: get_kv(&a.apikey, "value").unwrap_or_default(),
            }
        }
        _ => Auth::None,
    }
}

fn get_kv(items: &Option<Vec<PostmanKv>>, key: &str) -> Option<String> {
    items
        .as_ref()?
        .iter()
        .find(|kv| kv.key == key)
        .map(|kv| kv.value.clone())
}

fn map_body(b: &PostmanBody) -> Body {
    match b.mode.as_deref() {
        Some("raw") => {
            let text = b.raw.clone().unwrap_or_default();
            let ct = b
                .options
                .as_ref()
                .and_then(|o| o.raw.as_ref())
                .and_then(|r| r.language.as_deref());
            if matches!(ct, Some("json")) || looks_like_json(&text) {
                Body::Json { text }
            } else {
                Body::Text {
                    content_type: "text/plain".into(),
                    text,
                }
            }
        }
        Some("urlencoded") => Body::Form {
            fields: b
                .urlencoded
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|kv| !kv.disabled.unwrap_or(false))
                .map(|kv| (kv.key.clone(), kv.value.clone().unwrap_or_default()))
                .collect(),
        },
        Some("formdata") => Body::Form {
            fields: b
                .formdata
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .filter(|kv| !kv.disabled.unwrap_or(false))
                .map(|kv| (kv.key.clone(), kv.value.clone().unwrap_or_default()))
                .collect(),
        },
        _ => Body::None,
    }
}

fn looks_like_json(s: &str) -> bool {
    let trimmed = s.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn parse_method(s: &str) -> Method {
    match s.to_ascii_uppercase().as_str() {
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "PATCH" => Method::Patch,
        "DELETE" => Method::Delete,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        _ => Method::Get,
    }
}

fn sanitize_path_segment(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}

pub fn slugify(s: &str) -> String {
    let v = sanitize_path_segment(s);
    if v.is_empty() {
        "request".into()
    } else {
        v.chars().take(80).collect()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Postman v2.1 schema (minimal subset)
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PostmanCollection {
    #[serde(default)]
    item: Vec<PostmanItem>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PostmanItem {
    Folder {
        #[serde(default)]
        name: String,
        item: Vec<PostmanItem>,
    },
    Request {
        #[serde(default)]
        name: String,
        request: PostmanRequest,
        #[serde(default)]
        event: Option<Vec<PostmanEvent>>,
    },
}

#[derive(Deserialize)]
struct PostmanRequest {
    method: Option<String>,
    url: Option<PostmanUrl>,
    #[serde(default)]
    header: Option<Vec<PostmanHeader>>,
    #[serde(default)]
    body: Option<PostmanBody>,
    #[serde(default)]
    auth: Option<PostmanAuth>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PostmanUrl {
    Raw(String),
    Object {
        #[serde(default)]
        raw: Option<String>,
        #[serde(default)]
        host: Option<Vec<String>>,
        #[serde(default)]
        path: Option<Vec<String>>,
        #[serde(default)]
        query: Option<Vec<PostmanQueryParam>>,
    },
}

#[derive(Deserialize)]
struct PostmanQueryParam {
    key: Option<String>,
    value: Option<String>,
    #[serde(default)]
    disabled: Option<bool>,
}

#[derive(Deserialize)]
struct PostmanHeader {
    key: String,
    value: String,
    #[serde(default)]
    disabled: Option<bool>,
}

#[derive(Deserialize)]
struct PostmanBody {
    mode: Option<String>,
    raw: Option<String>,
    urlencoded: Option<Vec<PostmanFormItem>>,
    formdata: Option<Vec<PostmanFormItem>>,
    options: Option<PostmanBodyOptions>,
}

#[derive(Deserialize)]
struct PostmanBodyOptions {
    raw: Option<PostmanRawOptions>,
}

#[derive(Deserialize)]
struct PostmanRawOptions {
    language: Option<String>,
}

#[derive(Deserialize)]
struct PostmanFormItem {
    key: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    disabled: Option<bool>,
}

#[derive(Deserialize)]
struct PostmanAuth {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    bearer: Option<Vec<PostmanKv>>,
    #[serde(default)]
    basic: Option<Vec<PostmanKv>>,
    #[serde(default)]
    apikey: Option<Vec<PostmanKv>>,
}

#[derive(Deserialize)]
struct PostmanKv {
    key: String,
    #[serde(default)]
    value: String,
}

#[derive(Deserialize)]
struct PostmanEvent {
    #[serde(default)]
    script: PostmanScript,
}

#[derive(Deserialize, Default)]
struct PostmanScript {
    #[serde(default)]
    exec: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_simple_get_with_query_and_headers() {
        let json = r#"{
            "info": {"name": "demo"},
            "item": [{
                "name": "List users",
                "request": {
                    "method": "GET",
                    "header": [{"key":"Accept","value":"application/json"}],
                    "url": {
                        "raw": "https://api.example.com/users?limit=10",
                        "host": ["api","example","com"],
                        "path": ["users"],
                        "query": [{"key":"limit","value":"10"}]
                    }
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert_eq!(imported.len(), 1);
        let r = &imported[0];
        assert_eq!(r.collection, "imported");
        assert_eq!(r.slug, "list-users");
        assert_eq!(r.def.name, "List users");
        assert_eq!(r.def.compose.method, Method::Get);
        assert_eq!(r.def.compose.url, "https://api.example.com/users?limit=10");
        assert_eq!(r.def.compose.query, vec![("limit".into(), "10".into())]);
        assert_eq!(
            r.def.compose.headers,
            vec![("Accept".into(), "application/json".into())]
        );
    }

    #[test]
    fn folder_name_becomes_collection() {
        let json = r#"{
            "info": {"name": "d"},
            "item": [{
                "name": "Users API",
                "item": [{
                    "name": "Get one",
                    "request": {"method":"GET","url":"https://x/y"}
                }]
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].collection, "users-api");
        assert_eq!(imported[0].slug, "get-one");
    }

    #[test]
    fn nested_folders_get_joined() {
        let json = r#"{
            "info": {"name": "d"},
            "item": [{
                "name": "v1",
                "item": [{
                    "name": "users",
                    "item": [{
                        "name": "list",
                        "request": {"method":"GET","url":"https://x"}
                    }]
                }]
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert_eq!(imported[0].collection, "v1-users");
    }

    #[test]
    fn maps_bearer_auth() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"GET",
                    "url":"https://x",
                    "auth":{"type":"bearer","bearer":[{"key":"token","value":"abc"}]}
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert!(matches!(&imported[0].def.compose.auth, Auth::Bearer { token } if token == "abc"));
    }

    #[test]
    fn maps_basic_auth() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"GET","url":"https://x",
                    "auth":{"type":"basic","basic":[
                        {"key":"username","value":"alice"},
                        {"key":"password","value":"s3cret"}
                    ]}
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert!(matches!(
            &imported[0].def.compose.auth,
            Auth::Basic { username, password } if username == "alice" && password == "s3cret"
        ));
    }

    #[test]
    fn maps_apikey_in_query() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"GET","url":"https://x",
                    "auth":{"type":"apikey","apikey":[
                        {"key":"key","value":"k"},
                        {"key":"value","value":"v"},
                        {"key":"in","value":"query"}
                    ]}
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert!(matches!(
            &imported[0].def.compose.auth,
            Auth::ApiKey { location: ApiKeyLocation::Query, name, value }
                if name == "k" && value == "v"
        ));
    }

    #[test]
    fn maps_raw_json_body() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"POST","url":"https://x",
                    "body":{"mode":"raw","raw":"{\"a\":1}",
                            "options":{"raw":{"language":"json"}}}
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert!(
            matches!(&imported[0].def.compose.body, Body::Json { text } if text == r#"{"a":1}"#)
        );
    }

    #[test]
    fn maps_urlencoded_body() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"POST","url":"https://x",
                    "body":{"mode":"urlencoded","urlencoded":[
                        {"key":"a","value":"1"},
                        {"key":"b","value":"2","disabled":true}
                    ]}
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        match &imported[0].def.compose.body {
            Body::Form { fields } => assert_eq!(fields, &vec![("a".into(), "1".into())]),
            other => panic!("unexpected body {other:?}"),
        }
    }

    #[test]
    fn disabled_headers_skipped() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{
                    "method":"GET","url":"https://x",
                    "header":[
                        {"key":"A","value":"1"},
                        {"key":"B","value":"2","disabled":true}
                    ]
                }
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert_eq!(
            imported[0].def.compose.headers,
            vec![("A".into(), "1".into())]
        );
    }

    #[test]
    fn scripts_dropped_with_description_marker() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{"method":"GET","url":"https://x","description":"my req"},
                "event":[{"listen":"test","script":{"exec":["pm.test('ok', () => true)"]}}]
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert!(imported[0].def.description.contains("scripts were dropped"));
    }

    #[test]
    fn raw_url_string_supported() {
        let json = r#"{
            "info":{"name":"d"},
            "item":[{
                "name":"x",
                "request":{"method":"GET","url":"https://example.com/foo"}
            }]
        }"#;
        let imported = from_postman_v21(json).unwrap();
        assert_eq!(imported[0].def.compose.url, "https://example.com/foo");
    }

    #[test]
    fn unknown_methods_default_to_get() {
        assert_eq!(parse_method("WEIRD"), Method::Get);
        assert_eq!(parse_method("post"), Method::Post);
    }

    #[test]
    fn empty_slug_falls_back() {
        assert_eq!(slugify(""), "request");
        assert_eq!(slugify("!!!"), "request");
    }
}
