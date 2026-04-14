use std::time::Instant;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Method {
    fn as_reqwest(self) -> reqwest::Method {
        match self {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: Method,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default, with = "opt_bytes_as_base64")]
    pub body: Option<Bytes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    #[serde(with = "bytes_as_base64")]
    pub body: Bytes,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("timeout")]
    Timeout,
}

impl From<reqwest::Error> for RequestError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            RequestError::Timeout
        } else if e.is_builder() {
            RequestError::InvalidUrl(e.to_string())
        } else {
            RequestError::Network(e.to_string())
        }
    }
}

pub async fn send(req: Request) -> Result<Response, RequestError> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("stubhouse/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(RequestError::from)?;

    let mut builder = client.request(req.method.as_reqwest(), &req.url);
    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    let started = Instant::now();
    let resp = builder.send().await?;
    let status = resp.status().as_u16();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = resp.bytes().await?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let size_bytes = body.len();

    Ok(Response { status, headers, body, elapsed_ms, size_bytes })
}

mod bytes_as_base64 {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &Bytes, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&STANDARD.encode(b))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
        let s = String::deserialize(d)?;
        STANDARD.decode(&s).map(Bytes::from).map_err(serde::de::Error::custom)
    }
}

mod opt_bytes_as_base64 {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &Option<Bytes>, s: S) -> Result<S::Ok, S::Error> {
        match b {
            Some(b) => s.serialize_some(&STANDARD.encode(b)),
            None => s.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Bytes>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        match opt {
            None => Ok(None),
            Some(s) => STANDARD
                .decode(&s)
                .map(|v| Some(Bytes::from(v)))
                .map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_ok_round_trip() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-test", "yes")
                    .set_body_string("hi"),
            )
            .mount(&server)
            .await;

        let resp = send(Request {
            method: Method::Get,
            url: format!("{}/hello", server.uri()),
            headers: vec![],
            body: None,
        })
        .await
        .expect("request succeeds");

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body.as_ref(), b"hi");
        assert_eq!(resp.size_bytes, 2);
        assert!(
            resp.headers.iter().any(|(k, v)| k.eq_ignore_ascii_case("x-test") && v == "yes"),
            "expected x-test header in {:?}",
            resp.headers
        );
    }

    #[tokio::test]
    async fn post_with_body_and_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/echo"))
            .respond_with(ResponseTemplate::new(201).set_body_string("created"))
            .mount(&server)
            .await;

        let resp = send(Request {
            method: Method::Post,
            url: format!("{}/echo", server.uri()),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some(Bytes::from_static(b"{\"n\":1}")),
        })
        .await
        .unwrap();

        assert_eq!(resp.status, 201);
        assert_eq!(resp.body.as_ref(), b"created");
    }

    #[tokio::test]
    async fn invalid_url_errors() {
        let err = send(Request {
            method: Method::Get,
            url: "not a url".into(),
            headers: vec![],
            body: None,
        })
        .await
        .unwrap_err();
        matches!(err, RequestError::InvalidUrl(_) | RequestError::Network(_));
    }
}
