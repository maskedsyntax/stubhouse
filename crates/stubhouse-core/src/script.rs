//! Sandboxed Rhai scripting support.
//!
//! This module is intentionally small: it embeds Rhai, applies conservative
//! limits, and exposes typed helpers that higher-level request/mock/test flows
//! can build on without sharing a raw engine everywhere.

use std::collections::HashMap;

use rhai::{Dynamic, Engine, EvalAltResult, Map, Scope, AST};
use thiserror::Error;

use crate::{
    compose::{Body, Compose},
    http::{Method, Response},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptContext {
    pub env: HashMap<String, String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptOutcome {
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script parse error: {0}")]
    Parse(String),
    #[error("script execution error: {0}")]
    Eval(String),
    #[error("script returned {actual}, expected {expected}")]
    Type { expected: String, actual: String },
    #[error("invalid request method from script: {0}")]
    InvalidMethod(String),
}

pub struct ScriptRuntime {
    engine: Engine,
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRuntime {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(50_000);
        engine.set_max_call_levels(32);
        engine.set_max_string_size(256 * 1024);
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(10_000);
        engine.register_fn("assert", script_assert);
        Self { engine }
    }

    pub fn compile(&self, source: &str) -> Result<AST, ScriptError> {
        self.engine
            .compile(source)
            .map_err(|e| ScriptError::Parse(e.to_string()))
    }

    pub fn run(&self, source: &str, context: &ScriptContext) -> Result<ScriptOutcome, ScriptError> {
        let ast = self.compile(source)?;
        self.run_ast(&ast, context)
    }

    pub fn run_ast(
        &self,
        ast: &AST,
        context: &ScriptContext,
    ) -> Result<ScriptOutcome, ScriptError> {
        let mut scope = scope_for(context);
        let _ = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, ast)
            .map_err(|e| ScriptError::Eval(e.to_string()))?;
        Ok(ScriptOutcome {
            variables: map_to_strings(scope.get_value("variables").unwrap_or_default()),
        })
    }

    pub fn eval_bool(&self, source: &str, context: &ScriptContext) -> Result<bool, ScriptError> {
        let ast = self.compile(source)?;
        let mut scope = scope_for(context);
        let value = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, &ast)
            .map_err(|e| ScriptError::Eval(e.to_string()))?;
        let actual = value.type_name().to_string();
        value.try_cast::<bool>().ok_or_else(|| ScriptError::Type {
            expected: "bool".into(),
            actual,
        })
    }

    pub fn run_pre_request(
        &self,
        source: &str,
        request: &Compose,
        context: &ScriptContext,
    ) -> Result<Compose, ScriptError> {
        let ast = self.compile(source)?;
        self.run_pre_request_ast(&ast, request, context)
    }

    pub fn run_pre_request_ast(
        &self,
        ast: &AST,
        request: &Compose,
        context: &ScriptContext,
    ) -> Result<Compose, ScriptError> {
        let mut scope = scope_for(context);
        scope.push("request", compose_to_map(request));
        let _ = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, ast)
            .map_err(|e| ScriptError::Eval(e.to_string()))?;
        let request_map = scope.get_value("request").unwrap_or_default();
        map_to_compose(request_map, request)
    }

    pub fn run_post_response(
        &self,
        source: &str,
        response: &Response,
        context: &ScriptContext,
    ) -> Result<ScriptOutcome, ScriptError> {
        let ast = self.compile(source)?;
        self.run_post_response_ast(&ast, response, context)
    }

    pub fn run_post_response_ast(
        &self,
        ast: &AST,
        response: &Response,
        context: &ScriptContext,
    ) -> Result<ScriptOutcome, ScriptError> {
        let mut scope = scope_for(context);
        scope.push("response", response_to_map(response));
        let _ = self
            .engine
            .eval_ast_with_scope::<Dynamic>(&mut scope, ast)
            .map_err(|e| ScriptError::Eval(e.to_string()))?;
        Ok(ScriptOutcome {
            variables: map_to_strings(scope.get_value("variables").unwrap_or_default()),
        })
    }
}

fn scope_for(context: &ScriptContext) -> Scope<'static> {
    let mut scope = Scope::new();
    scope.push("env", strings_to_map(&context.env));
    scope.push("variables", strings_to_map(&context.variables));
    scope
}

fn strings_to_map(values: &HashMap<String, String>) -> Map {
    values
        .iter()
        .map(|(key, value)| (key.clone().into(), Dynamic::from(value.clone())))
        .collect()
}

fn map_to_strings(values: Map) -> HashMap<String, String> {
    values
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn compose_to_map(request: &Compose) -> Map {
    let mut map = Map::new();
    map.insert(
        "method".into(),
        Dynamic::from(method_to_string(request.method)),
    );
    map.insert("url".into(), Dynamic::from(request.url.clone()));
    map.insert(
        "headers".into(),
        Dynamic::from(pairs_to_map(&request.headers)),
    );
    map.insert("query".into(), Dynamic::from(pairs_to_map(&request.query)));
    map.insert("body".into(), Dynamic::from(body_to_string(&request.body)));
    map
}

fn map_to_compose(map: Map, fallback: &Compose) -> Result<Compose, ScriptError> {
    let method = map
        .get("method")
        .map(|value| parse_method(&value.to_string()))
        .transpose()?
        .unwrap_or(fallback.method);
    let url = map
        .get("url")
        .map(|value| value.to_string())
        .unwrap_or_else(|| fallback.url.clone());
    let headers = map
        .get("headers")
        .and_then(|value| value.clone().try_cast::<Map>())
        .map(map_to_pairs)
        .unwrap_or_else(|| fallback.headers.clone());
    let query = map
        .get("query")
        .and_then(|value| value.clone().try_cast::<Map>())
        .map(map_to_pairs)
        .unwrap_or_else(|| fallback.query.clone());
    let body = map
        .get("body")
        .map(|value| value.to_string())
        .map(|text| body_from_script_text(text, &fallback.body))
        .unwrap_or_else(|| fallback.body.clone());

    Ok(Compose {
        method,
        url,
        query,
        headers,
        auth: fallback.auth.clone(),
        body,
    })
}

fn pairs_to_map(pairs: &[(String, String)]) -> Map {
    pairs
        .iter()
        .map(|(key, value)| (key.clone().into(), Dynamic::from(value.clone())))
        .collect()
}

fn map_to_pairs(map: Map) -> Vec<(String, String)> {
    map.into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn body_to_string(body: &Body) -> String {
    match body {
        Body::None => String::new(),
        Body::Text { text, .. } | Body::Json { text } => text.clone(),
        Body::Form { fields } => serde_urlencoded::to_string(fields).unwrap_or_default(),
    }
}

fn body_from_script_text(text: String, fallback: &Body) -> Body {
    match fallback {
        Body::Json { .. } => Body::Json { text },
        Body::Text { content_type, .. } => Body::Text {
            content_type: content_type.clone(),
            text,
        },
        Body::Form { .. } => Body::Text {
            content_type: "text/plain; charset=utf-8".into(),
            text,
        },
        Body::None if text.is_empty() => Body::None,
        Body::None => Body::Text {
            content_type: "text/plain; charset=utf-8".into(),
            text,
        },
    }
}

fn parse_method(method: &str) -> Result<Method, ScriptError> {
    match method.to_ascii_uppercase().as_str() {
        "GET" => Ok(Method::Get),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "PATCH" => Ok(Method::Patch),
        "DELETE" => Ok(Method::Delete),
        "HEAD" => Ok(Method::Head),
        "OPTIONS" => Ok(Method::Options),
        other => Err(ScriptError::InvalidMethod(other.into())),
    }
}

fn method_to_string(method: Method) -> String {
    match method {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
    }
    .into()
}

fn response_to_map(response: &Response) -> Map {
    let mut map = Map::new();
    map.insert("status".into(), Dynamic::from(response.status as i64));
    map.insert(
        "headers".into(),
        Dynamic::from(pairs_to_map(&response.headers)),
    );
    map.insert(
        "body".into(),
        Dynamic::from(String::from_utf8_lossy(&response.body).into_owned()),
    );
    map.insert("time_ms".into(), Dynamic::from(response.elapsed_ms as i64));
    map.insert(
        "elapsed_ms".into(),
        Dynamic::from(response.elapsed_ms as i64),
    );
    map.insert(
        "size_bytes".into(),
        Dynamic::from(response.size_bytes as i64),
    );
    map
}

fn script_assert(condition: bool, message: &str) -> Result<(), Box<EvalAltResult>> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::Auth;
    use bytes::Bytes;

    fn context() -> ScriptContext {
        ScriptContext {
            env: HashMap::from([("token".into(), "abc".into())]),
            variables: HashMap::from([("count".into(), "1".into())]),
        }
    }

    fn request() -> Compose {
        Compose {
            method: Method::Post,
            url: "https://example.com/users".into(),
            query: vec![("page".into(), "1".into())],
            headers: vec![("Content-Type".into(), "application/json".into())],
            auth: Auth::None,
            body: Body::Json {
                text: r#"{"name":"Alice"}"#.into(),
            },
        }
    }

    fn response() -> Response {
        Response {
            status: 201,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Bytes::from_static(br#"{"token":"abc"}"#),
            elapsed_ms: 42,
            size_bytes: 15,
        }
    }

    #[test]
    fn eval_bool_returns_boolean_result() {
        let runtime = ScriptRuntime::new();
        assert!(runtime
            .eval_bool("env[\"token\"] == \"abc\"", &context())
            .unwrap());
    }

    #[test]
    fn eval_bool_rejects_non_boolean_result() {
        let runtime = ScriptRuntime::new();
        let err = runtime.eval_bool("40 + 2", &context()).unwrap_err();
        assert!(matches!(err, ScriptError::Type { .. }));
    }

    #[test]
    fn run_allows_variable_mutation() {
        let runtime = ScriptRuntime::new();
        let outcome = runtime
            .run(
                r#"
                variables["count"] = "2";
                variables["token"] = env["token"];
                "#,
                &context(),
            )
            .unwrap();
        assert_eq!(outcome.variables["count"], "2");
        assert_eq!(outcome.variables["token"], "abc");
    }

    #[test]
    fn assert_function_fails_script() {
        let runtime = ScriptRuntime::new();
        let err = runtime
            .run(r#"assert(false, "expected failure");"#, &context())
            .unwrap_err();
        assert!(err.to_string().contains("expected failure"));
    }

    #[test]
    fn syntax_errors_are_parse_errors() {
        let runtime = ScriptRuntime::new();
        let err = runtime.run("let =", &context()).unwrap_err();
        assert!(matches!(err, ScriptError::Parse(_)));
    }

    #[test]
    fn pre_request_script_mutates_request_fields() {
        let runtime = ScriptRuntime::new();
        let out = runtime
            .run_pre_request(
                r#"
                request.method = "PUT";
                request.url = request.url + "/42";
                request.query["debug"] = "true";
                request.headers["X-Token"] = env["token"];
                request.body = "{\"name\":\"Bob\"}";
                "#,
                &request(),
                &context(),
            )
            .unwrap();
        assert_eq!(out.method, Method::Put);
        assert_eq!(out.url, "https://example.com/users/42");
        assert!(out.query.contains(&("debug".into(), "true".into())));
        assert!(out.headers.contains(&("X-Token".into(), "abc".into())));
        assert!(matches!(&out.body, Body::Json { text } if text == r#"{"name":"Bob"}"#));
    }

    #[test]
    fn pre_request_script_rejects_unknown_method() {
        let runtime = ScriptRuntime::new();
        let err = runtime
            .run_pre_request(r#"request.method = "TRACE";"#, &request(), &context())
            .unwrap_err();
        assert!(matches!(err, ScriptError::InvalidMethod(method) if method == "TRACE"));
    }

    #[test]
    fn post_response_script_can_assert_and_extract_variables() {
        let runtime = ScriptRuntime::new();
        let outcome = runtime
            .run_post_response(
                r#"
                assert(response.status == 201, "expected created");
                assert(response.headers["Content-Type"] == "application/json", "expected json");
                variables["token"] = response.body;
                variables["elapsed"] = response.elapsed_ms.to_string();
                "#,
                &response(),
                &context(),
            )
            .unwrap();
        assert_eq!(outcome.variables["token"], r#"{"token":"abc"}"#);
        assert_eq!(outcome.variables["elapsed"], "42");
    }

    #[test]
    fn post_response_assertion_failure_is_error() {
        let runtime = ScriptRuntime::new();
        let err = runtime
            .run_post_response(
                r#"assert(response.status == 200, "expected ok");"#,
                &response(),
                &context(),
            )
            .unwrap_err();
        assert!(err.to_string().contains("expected ok"));
    }
}
