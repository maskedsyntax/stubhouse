use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::http::{Method, Request};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Auth {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey {
        #[serde(rename = "in")]
        location: ApiKeyLocation,
        name: String,
        value: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Body {
    None,
    Text { content_type: String, text: String },
    Json { text: String },
    Form { fields: Vec<(String, String)> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Compose {
    pub method: Method,
    pub url: String,
    #[serde(default)]
    pub query: Vec<(String, String)>,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default = "default_auth")]
    pub auth: Auth,
    #[serde(default = "default_body")]
    pub body: Body,
}

fn default_auth() -> Auth { Auth::None }
fn default_body() -> Body { Body::None }

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("invalid JSON body: {0}")]
    InvalidJson(String),
}

impl Compose {
    pub fn build(self) -> Result<Request, ComposeError> {
        let Compose { method, url, query, mut headers, auth, body } = self;

        let mut parsed = url::Url::parse(&url)
            .map_err(|e| ComposeError::InvalidUrl(e.to_string()))?;

        for (k, v) in query {
            if !k.is_empty() {
                parsed.query_pairs_mut().append_pair(&k, &v);
            }
        }

        match auth {
            Auth::None => {}
            Auth::Bearer { token } => {
                headers.push(("Authorization".into(), format!("Bearer {token}")));
            }
            Auth::Basic { username, password } => {
                use base64::{engine::general_purpose::STANDARD, Engine as _};
                let creds = STANDARD.encode(format!("{username}:{password}"));
                headers.push(("Authorization".into(), format!("Basic {creds}")));
            }
            Auth::ApiKey { location, name, value } => match location {
                ApiKeyLocation::Header => {
                    headers.push((name, value));
                }
                ApiKeyLocation::Query => {
                    parsed.query_pairs_mut().append_pair(&name, &value);
                }
            },
        }

        let (body_bytes, body_ct) = match body {
            Body::None => (None, None),
            Body::Text { content_type, text } => (
                Some(Bytes::from(text.into_bytes())),
                Some(content_type),
            ),
            Body::Json { text } => {
                if !text.trim().is_empty() {
                    serde_json::from_str::<serde_json::Value>(&text)
                        .map_err(|e| ComposeError::InvalidJson(e.to_string()))?;
                }
                (Some(Bytes::from(text.into_bytes())), Some("application/json".into()))
            }
            Body::Form { fields } => {
                let encoded = serde_urlencoded::to_string(&fields)
                    .map_err(|e| ComposeError::InvalidJson(e.to_string()))?;
                (
                    Some(Bytes::from(encoded.into_bytes())),
                    Some("application/x-www-form-urlencoded".into()),
                )
            }
        };

        if let Some(ct) = body_ct {
            let already = headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
            if !already {
                headers.push(("Content-Type".into(), ct));
            }
        }

        Ok(Request {
            method,
            url: parsed.to_string(),
            headers,
            body: body_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Compose {
        Compose {
            method: Method::Get,
            url: "https://example.com/api".into(),
            query: vec![],
            headers: vec![],
            auth: Auth::None,
            body: Body::None,
        }
    }

    #[test]
    fn appends_query_params() {
        let req = Compose {
            query: vec![("q".into(), "hello world".into()), ("page".into(), "2".into())],
            ..base()
        }.build().unwrap();
        assert!(req.url.contains("q=hello+world") || req.url.contains("q=hello%20world"));
        assert!(req.url.contains("page=2"));
    }

    #[test]
    fn bearer_auth_adds_authorization_header() {
        let req = Compose { auth: Auth::Bearer { token: "abc.def".into() }, ..base() }
            .build().unwrap();
        let h = req.headers.iter().find(|(k, _)| k == "Authorization").unwrap();
        assert_eq!(h.1, "Bearer abc.def");
    }

    #[test]
    fn basic_auth_base64_encodes_credentials() {
        let req = Compose {
            auth: Auth::Basic { username: "alice".into(), password: "s3cret".into() },
            ..base()
        }.build().unwrap();
        let h = req.headers.iter().find(|(k, _)| k == "Authorization").unwrap();
        assert_eq!(h.1, "Basic YWxpY2U6czNjcmV0");
    }

    #[test]
    fn api_key_in_header() {
        let req = Compose {
            auth: Auth::ApiKey {
                location: ApiKeyLocation::Header,
                name: "X-Api-Key".into(),
                value: "kkk".into(),
            },
            ..base()
        }.build().unwrap();
        assert!(req.headers.iter().any(|(k, v)| k == "X-Api-Key" && v == "kkk"));
    }

    #[test]
    fn api_key_in_query() {
        let req = Compose {
            auth: Auth::ApiKey {
                location: ApiKeyLocation::Query,
                name: "key".into(),
                value: "kkk".into(),
            },
            ..base()
        }.build().unwrap();
        assert!(req.url.contains("key=kkk"));
    }

    #[test]
    fn json_body_sets_content_type_and_validates() {
        let req = Compose {
            method: Method::Post,
            body: Body::Json { text: r#"{"a":1}"#.into() },
            ..base()
        }.build().unwrap();
        assert_eq!(req.body.as_deref(), Some(br#"{"a":1}"#.as_ref()));
        let ct = req.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type")).unwrap();
        assert_eq!(ct.1, "application/json");
    }

    #[test]
    fn invalid_json_body_errors() {
        let err = Compose {
            method: Method::Post,
            body: Body::Json { text: "not json".into() },
            ..base()
        }.build().unwrap_err();
        matches!(err, ComposeError::InvalidJson(_));
    }

    #[test]
    fn form_body_url_encodes() {
        let req = Compose {
            method: Method::Post,
            body: Body::Form {
                fields: vec![("a".into(), "1".into()), ("b".into(), "hello world".into())],
            },
            ..base()
        }.build().unwrap();
        assert_eq!(req.body.as_deref(), Some(b"a=1&b=hello+world".as_ref()));
        let ct = req.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type")).unwrap();
        assert_eq!(ct.1, "application/x-www-form-urlencoded");
    }

    #[test]
    fn explicit_content_type_header_not_overridden() {
        let req = Compose {
            method: Method::Post,
            headers: vec![("content-type".into(), "application/vnd.custom+json".into())],
            body: Body::Json { text: r#"{"a":1}"#.into() },
            ..base()
        }.build().unwrap();
        let cts: Vec<_> = req
            .headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .collect();
        assert_eq!(cts.len(), 1);
        assert_eq!(cts[0].1, "application/vnd.custom+json");
    }

    #[test]
    fn invalid_url_errors() {
        let err = Compose { url: "not a url".into(), ..base() }.build().unwrap_err();
        matches!(err, ComposeError::InvalidUrl(_));
    }
}
