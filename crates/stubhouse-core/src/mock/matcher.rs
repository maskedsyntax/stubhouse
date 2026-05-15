//! Path matcher for mock routes.
//!
//! Supports exact segments, `:name` parameter captures, `*` single-segment
//! wildcards, and `**` multi-segment wildcards. Among matching rules, higher
//! explicit priority wins first; ties are resolved by route specificity:
//! exact > `:param` > `*` > `**` > catch-all.

use std::collections::HashMap;

use super::MockRule;
use crate::http::Method;

#[derive(Debug, Clone)]
pub struct Match<'r> {
    pub rule: &'r MockRule,
    pub params: HashMap<String, String>,
}

pub fn find_match<'r>(rules: &'r [MockRule], method: Method, path: &str) -> Option<Match<'r>> {
    let request_segments: Vec<&str> = split_segments(path);
    let mut best: Option<(Match<'r>, RouteScore)> = None;

    for (index, rule) in rules.iter().enumerate() {
        if rule.method != method {
            continue;
        }

        if let Some((params, score)) = match_route(rule, &request_segments, index) {
            let candidate = Match { rule, params };
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((candidate, score)),
            }
        }
    }

    best.map(|(m, _)| m)
}

fn split_segments(path: &str) -> Vec<&str> {
    path.trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RouteScore {
    priority: i32,
    specificity: Vec<u8>,
    literal_count: usize,
    segment_count: usize,
    reverse_index: usize,
}

fn match_route(
    rule: &MockRule,
    request_segments: &[&str],
    index: usize,
) -> Option<(HashMap<String, String>, RouteScore)> {
    let rule_segments = split_segments(&rule.path);
    let mut params = HashMap::new();
    if !match_segments(&rule_segments, request_segments, &mut params) {
        return None;
    }

    let specificity = rule_segments.iter().map(|s| segment_weight(s)).collect();
    let literal_count = rule_segments
        .iter()
        .filter(|s| !is_param(s) && **s != "*" && **s != "**")
        .count();

    Some((
        params,
        RouteScore {
            priority: rule.priority,
            specificity,
            literal_count,
            segment_count: rule_segments.len(),
            // Earlier rules win when every route trait ties.
            reverse_index: usize::MAX - index,
        },
    ))
}

fn match_segments<'a>(
    rule_segments: &[&str],
    request_segments: &[&'a str],
    params: &mut HashMap<String, String>,
) -> bool {
    match (rule_segments.split_first(), request_segments.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((&"**", rest_rule)), _) => {
            for skip in 0..=request_segments.len() {
                let mut branch_params = params.clone();
                if match_segments(rest_rule, &request_segments[skip..], &mut branch_params) {
                    *params = branch_params;
                    return true;
                }
            }
            false
        }
        (Some((&rule_segment, rest_rule)), Some((request_segment, rest_request))) => {
            if rule_segment == "*" {
                return match_segments(rest_rule, rest_request, params);
            }
            if let Some(name) = rule_segment.strip_prefix(':') {
                params.insert(name.to_string(), (*request_segment).to_string());
                return match_segments(rest_rule, rest_request, params);
            }
            rule_segment == *request_segment && match_segments(rest_rule, rest_request, params)
        }
        (Some(_), None) => false,
    }
}

fn segment_weight(segment: &str) -> u8 {
    if segment == "**" {
        1
    } else if segment == "*" {
        2
    } else if is_param(segment) {
        3
    } else {
        4
    }
}

fn is_param(segment: &str) -> bool {
    segment.starts_with(':') && segment.len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockBody, MockResponse};

    fn rule(method: Method, path: &str) -> MockRule {
        MockRule {
            name: format!("{method:?} {path}"),
            method,
            path: path.into(),
            priority: 0,
            response: MockResponse {
                status: 200,
                headers: vec![],
                body: MockBody::None,
                delay_ms: 0,
            },
            scenarios: vec![],
        }
    }

    #[test]
    fn exact_match() {
        let rules = vec![rule(Method::Get, "/users")];
        let m = find_match(&rules, Method::Get, "/users").unwrap();
        assert_eq!(m.rule.path, "/users");
        assert!(m.params.is_empty());
    }

    #[test]
    fn method_filter() {
        let rules = vec![rule(Method::Post, "/users")];
        assert!(find_match(&rules, Method::Get, "/users").is_none());
    }

    #[test]
    fn single_path_param() {
        let rules = vec![rule(Method::Get, "/users/:id")];
        let m = find_match(&rules, Method::Get, "/users/42").unwrap();
        assert_eq!(m.params.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn multiple_path_params() {
        let rules = vec![rule(Method::Get, "/users/:uid/posts/:pid")];
        let m = find_match(&rules, Method::Get, "/users/u1/posts/p9").unwrap();
        assert_eq!(m.params.get("uid"), Some(&"u1".to_string()));
        assert_eq!(m.params.get("pid"), Some(&"p9".to_string()));
    }

    #[test]
    fn different_segment_counts_dont_match() {
        let rules = vec![rule(Method::Get, "/users/:id")];
        assert!(find_match(&rules, Method::Get, "/users").is_none());
        assert!(find_match(&rules, Method::Get, "/users/1/extra").is_none());
    }

    #[test]
    fn exact_beats_param_when_both_listed_first() {
        let rules = vec![
            rule(Method::Get, "/users/me"),  // exact first
            rule(Method::Get, "/users/:id"), // param fallback
        ];
        let m = find_match(&rules, Method::Get, "/users/me").unwrap();
        assert_eq!(m.rule.path, "/users/me");
        assert!(m.params.is_empty());
    }

    #[test]
    fn trailing_slash_tolerated() {
        let rules = vec![rule(Method::Get, "/users")];
        assert!(find_match(&rules, Method::Get, "/users/").is_some());
        assert!(find_match(&rules, Method::Get, "users").is_some());
    }

    #[test]
    fn no_match_returns_none() {
        let rules = vec![rule(Method::Get, "/a")];
        assert!(find_match(&rules, Method::Get, "/b").is_none());
    }

    #[test]
    fn exact_beats_param_even_when_listed_later() {
        let rules = vec![
            rule(Method::Get, "/users/:id"),
            rule(Method::Get, "/users/me"),
        ];
        let m = find_match(&rules, Method::Get, "/users/me").unwrap();
        assert_eq!(m.rule.path, "/users/me");
        assert!(m.params.is_empty());
    }

    #[test]
    fn param_beats_single_segment_wildcard() {
        let rules = vec![
            rule(Method::Get, "/files/*"),
            rule(Method::Get, "/files/:name"),
        ];
        let m = find_match(&rules, Method::Get, "/files/report").unwrap();
        assert_eq!(m.rule.path, "/files/:name");
        assert_eq!(m.params.get("name"), Some(&"report".to_string()));
    }

    #[test]
    fn single_segment_wildcard_matches_one_segment_only() {
        let rules = vec![rule(Method::Get, "/files/*")];
        assert!(find_match(&rules, Method::Get, "/files/readme").is_some());
        assert!(find_match(&rules, Method::Get, "/files/a/b").is_none());
    }

    #[test]
    fn globstar_matches_zero_or_more_segments() {
        let rules = vec![rule(Method::Get, "/assets/**")];
        assert!(find_match(&rules, Method::Get, "/assets").is_some());
        assert!(find_match(&rules, Method::Get, "/assets/a/b/c").is_some());
    }

    #[test]
    fn prefixed_globstar_beats_plain_catch_all() {
        let rules = vec![rule(Method::Get, "/**"), rule(Method::Get, "/api/**")];
        let m = find_match(&rules, Method::Get, "/api/users/1").unwrap();
        assert_eq!(m.rule.path, "/api/**");
    }

    #[test]
    fn priority_breaks_specificity_ties() {
        let mut low = rule(Method::Get, "/users/:id");
        low.priority = 1;
        let mut high = rule(Method::Get, "/users/:name");
        high.priority = 2;
        let rules = vec![low, high];
        let m = find_match(&rules, Method::Get, "/users/42").unwrap();
        assert_eq!(m.rule.path, "/users/:name");
    }
}
