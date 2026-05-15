use std::collections::HashMap;

use crate::compose::{Auth, Body, Compose};

pub fn interpolate_compose(compose: &Compose, vars: &HashMap<String, String>) -> Compose {
    Compose {
        method: compose.method,
        url: interpolate_string(&compose.url, vars),
        query: compose
            .query
            .iter()
            .map(|(k, v)| (interpolate_string(k, vars), interpolate_string(v, vars)))
            .collect(),
        headers: compose
            .headers
            .iter()
            .map(|(k, v)| (interpolate_string(k, vars), interpolate_string(v, vars)))
            .collect(),
        auth: interpolate_auth(&compose.auth, vars),
        body: interpolate_body(&compose.body, vars),
    }
}

pub fn interpolate_string(s: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let key = after_open[..end].trim();
            if let Some(val) = resolve_variable(key, vars) {
                result.push_str(&val);
            } else {
                result.push_str(&rest[start..start + 2 + end + 2]);
            }
            rest = &after_open[end + 2..];
        } else {
            result.push_str(&rest[start..]);
            rest = "";
        }
    }
    result.push_str(rest);
    result
}

fn resolve_variable(key: &str, vars: &HashMap<String, String>) -> Option<String> {
    if let Some(builtin) = key.strip_prefix('$') {
        return resolve_builtin(builtin);
    }
    vars.get(key).cloned()
}

fn resolve_builtin(name: &str) -> Option<String> {
    match name {
        "timestamp" => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Some(ts.to_string())
        }
        "isoTimestamp" => {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let secs = ts as i64;
            let days = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;

            let (year, month, day) = days_to_ymd(days + 719_468);
            Some(format!(
                "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z"
            ))
        }
        "randomUUID" => Some(uuid_v4()),
        _ => {
            if let Some(var_name) = name.strip_prefix("env.") {
                std::env::var(var_name).ok()
            } else {
                None
            }
        }
    }
}

fn days_to_ymd(day_count: i64) -> (i64, u32, u32) {
    let era = if day_count >= 0 {
        day_count
    } else {
        day_count - 146_096 + 1
    } / 146_097;
    let doe = (day_count - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    getrandom(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

fn getrandom(buf: &mut [u8]) {
    use std::fs::File;
    use std::io::Read;
    if let Ok(mut f) = File::open("/dev/urandom") {
        let _ = f.read_exact(buf);
    }
}

fn interpolate_auth(auth: &Auth, vars: &HashMap<String, String>) -> Auth {
    match auth {
        Auth::None => Auth::None,
        Auth::Bearer { token } => Auth::Bearer {
            token: interpolate_string(token, vars),
        },
        Auth::Basic { username, password } => Auth::Basic {
            username: interpolate_string(username, vars),
            password: interpolate_string(password, vars),
        },
        Auth::ApiKey {
            location,
            name,
            value,
        } => Auth::ApiKey {
            location: *location,
            name: interpolate_string(name, vars),
            value: interpolate_string(value, vars),
        },
    }
}

fn interpolate_body(body: &Body, vars: &HashMap<String, String>) -> Body {
    match body {
        Body::None => Body::None,
        Body::Json { text } => Body::Json {
            text: interpolate_string(text, vars),
        },
        Body::Text { content_type, text } => Body::Text {
            content_type: interpolate_string(content_type, vars),
            text: interpolate_string(text, vars),
        },
        Body::Form { fields } => Body::Form {
            fields: fields
                .iter()
                .map(|(k, v)| (interpolate_string(k, vars), interpolate_string(v, vars)))
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("base_url".into(), "http://localhost:3000".into());
        m.insert("token".into(), "abc123".into());
        m.insert("user".into(), "alice".into());
        m
    }

    #[test]
    fn basic_substitution() {
        assert_eq!(
            interpolate_string("{{base_url}}/api/users", &vars()),
            "http://localhost:3000/api/users"
        );
    }

    #[test]
    fn multiple_variables() {
        assert_eq!(
            interpolate_string("{{base_url}}/{{user}}", &vars()),
            "http://localhost:3000/alice"
        );
    }

    #[test]
    fn missing_variable_kept_verbatim() {
        assert_eq!(interpolate_string("{{missing}}", &vars()), "{{missing}}");
    }

    #[test]
    fn no_placeholders_unchanged() {
        assert_eq!(interpolate_string("hello world", &vars()), "hello world");
    }

    #[test]
    fn unclosed_placeholder_kept() {
        assert_eq!(interpolate_string("{{base_url", &vars()), "{{base_url");
    }

    #[test]
    fn whitespace_trimmed_in_key() {
        assert_eq!(
            interpolate_string("{{ base_url }}/path", &vars()),
            "http://localhost:3000/path"
        );
    }

    #[test]
    fn builtin_timestamp_is_numeric() {
        let result = interpolate_string("{{$timestamp}}", &HashMap::new());
        assert!(
            result.parse::<u64>().is_ok(),
            "expected numeric timestamp, got: {result}"
        );
    }

    #[test]
    fn builtin_iso_timestamp_format() {
        let result = interpolate_string("{{$isoTimestamp}}", &HashMap::new());
        assert!(
            result.contains('T') && result.ends_with('Z'),
            "unexpected format: {result}"
        );
    }

    #[test]
    fn builtin_uuid_format() {
        let result = interpolate_string("{{$randomUUID}}", &HashMap::new());
        assert_eq!(result.len(), 36);
        assert_eq!(result.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn interpolate_compose_applies_to_all_fields() {
        use crate::http::Method;
        let compose = Compose {
            method: Method::Post,
            url: "{{base_url}}/users".into(),
            query: vec![("token".into(), "{{token}}".into())],
            headers: vec![("Authorization".into(), "Bearer {{token}}".into())],
            auth: Auth::Bearer {
                token: "{{token}}".into(),
            },
            body: Body::Json {
                text: r#"{"user": "{{user}}"}"#.into(),
            },
        };
        let result = interpolate_compose(&compose, &vars());
        assert_eq!(result.url, "http://localhost:3000/users");
        assert_eq!(result.query[0].1, "abc123");
        assert_eq!(result.headers[0].1, "Bearer abc123");
        assert!(matches!(&result.auth, Auth::Bearer { token } if token == "abc123"));
        assert!(matches!(&result.body, Body::Json { text } if text.contains("alice")));
    }
}
