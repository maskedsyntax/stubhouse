use crate::compose::{Compose, ComposeError};
use crate::http::{Method, Request};

pub fn to_curl(compose: &Compose) -> Result<String, ComposeError> {
    let req = compose.clone().build()?;
    Ok(curl_from_request(&req))
}

fn curl_from_request(req: &Request) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push("curl".into());

    if !matches!(req.method, Method::Get) {
        parts.push("-X".into());
        parts.push(method_str(req.method).into());
    }

    for (k, v) in &req.headers {
        parts.push("-H".into());
        parts.push(shell_quote(&format!("{k}: {v}")));
    }

    if let Some(body) = req.body.as_ref() {
        if !body.is_empty() {
            parts.push("--data-binary".into());
            // Body is a JSON/text/form payload — stringify with lossy UTF-8.
            // For true binary bodies users will need the file-upload path (post-v1).
            let s = String::from_utf8_lossy(body);
            parts.push(shell_quote(&s));
        }
    }

    parts.push(shell_quote(&req.url));
    parts.join(" ")
}

fn method_str(m: Method) -> &'static str {
    match m {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
    }
}

/// POSIX-style single-quote escape. Safe for sh/bash/zsh.
fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | ',' | '=' | '@')
        })
    {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::{Auth, Body, Compose};

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
    fn simple_get_omits_method_flag() {
        let s = to_curl(&base()).unwrap();
        assert_eq!(s, "curl https://example.com/api");
    }

    #[test]
    fn post_emits_method_flag() {
        let c = Compose {
            method: Method::Post,
            ..base()
        };
        let s = to_curl(&c).unwrap();
        assert!(s.contains("-X POST"));
    }

    #[test]
    fn headers_are_single_quoted() {
        let c = Compose {
            headers: vec![("X-Custom".into(), "hello world".into())],
            ..base()
        };
        let s = to_curl(&c).unwrap();
        assert!(s.contains("-H 'X-Custom: hello world'"), "got: {s}");
    }

    #[test]
    fn bearer_auth_appears_as_header() {
        let c = Compose {
            auth: Auth::Bearer {
                token: "abc.def".into(),
            },
            ..base()
        };
        let s = to_curl(&c).unwrap();
        assert!(s.contains("Authorization: Bearer abc.def"));
    }

    #[test]
    fn json_body_is_quoted_with_data_binary() {
        let c = Compose {
            method: Method::Post,
            body: Body::Json {
                text: r#"{"a":1,"b":"x y"}"#.into(),
            },
            ..base()
        };
        let s = to_curl(&c).unwrap();
        assert!(s.contains("--data-binary"), "got: {s}");
        assert!(s.contains(r#"'{"a":1,"b":"x y"}'"#), "got: {s}");
        assert!(s.contains("Content-Type: application/json"));
    }

    #[test]
    fn single_quotes_inside_body_are_escaped() {
        let c = Compose {
            method: Method::Post,
            body: Body::Json {
                text: r#"{"q":"O'Brien"}"#.into(),
            },
            ..base()
        };
        let s = to_curl(&c).unwrap();
        // POSIX trick: close, escape, reopen.
        assert!(s.contains(r#"O'\''Brien"#), "got: {s}");
    }

    #[test]
    fn query_params_round_trip_into_url() {
        let c = Compose {
            query: vec![("q".into(), "hello world".into())],
            ..base()
        };
        let s = to_curl(&c).unwrap();
        assert!(
            s.contains("q=hello+world") || s.contains("q=hello%20world"),
            "got: {s}"
        );
    }

    #[test]
    fn plain_url_is_not_quoted() {
        let s = to_curl(&base()).unwrap();
        assert!(s.ends_with(" https://example.com/api"), "got: {s}");
    }
}
