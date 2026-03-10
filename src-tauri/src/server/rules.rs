use crate::models::MatchRule;
use serde_json::Value;
use std::collections::HashMap;

/// Returns true if all rules match. Empty rules list always matches.
pub fn evaluate_rules(
    rules: &[MatchRule],
    headers: &HashMap<String, String>,
    query: &HashMap<String, String>,
    body_json: Option<&Value>,
    path_params: &HashMap<String, String>,
) -> bool {
    rules
        .iter()
        .all(|rule| evaluate_rule(rule, headers, query, body_json, path_params))
}

fn get_value(
    rule: &MatchRule,
    headers: &HashMap<String, String>,
    query: &HashMap<String, String>,
    body_json: Option<&Value>,
    path_params: &HashMap<String, String>,
) -> Option<String> {
    match rule.source.as_str() {
        "header" => headers
            .get(&rule.key)
            .cloned()
            .or_else(|| headers.get(&rule.key.to_lowercase()).cloned()),
        "query" => query.get(&rule.key).cloned(),
        "path_param" => path_params.get(&rule.key).cloned(),
        "body" => {
            if let Some(json) = body_json {
                if let Some(v) = json.get(&rule.key) {
                    return Some(match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn evaluate_rule(
    rule: &MatchRule,
    headers: &HashMap<String, String>,
    query: &HashMap<String, String>,
    body_json: Option<&Value>,
    path_params: &HashMap<String, String>,
) -> bool {
    let actual = get_value(rule, headers, query, body_json, path_params);
    match rule.operator.as_str() {
        "exists" => actual.is_some(),
        "not_exists" => actual.is_none(),
        "eq" => actual.as_deref() == rule.value.as_deref(),
        "contains" => match (&actual, &rule.value) {
            (Some(a), Some(v)) => a.contains(v.as_str()),
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn query(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn rule(source: &str, key: &str, operator: &str, value: Option<&str>) -> MatchRule {
        MatchRule {
            source: source.to_string(),
            key: key.to_string(),
            operator: operator.to_string(),
            value: value.map(|s| s.to_string()),
        }
    }

    #[test]
    fn empty_rules_always_matches() {
        assert!(evaluate_rules(
            &[],
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new()
        ));
    }

    #[test]
    fn header_eq_matches() {
        let rules = vec![rule("header", "Authorization", "eq", Some("Bearer token123"))];
        let h = headers(&[("Authorization", "Bearer token123")]);
        assert!(evaluate_rules(&rules, &h, &HashMap::new(), None, &HashMap::new()));
    }

    #[test]
    fn header_eq_no_match() {
        let rules = vec![rule("header", "Authorization", "eq", Some("Bearer other"))];
        let h = headers(&[("Authorization", "Bearer token123")]);
        assert!(!evaluate_rules(&rules, &h, &HashMap::new(), None, &HashMap::new()));
    }

    #[test]
    fn header_exists_matches() {
        let rules = vec![rule("header", "Authorization", "exists", None)];
        let h = headers(&[("Authorization", "anything")]);
        assert!(evaluate_rules(&rules, &h, &HashMap::new(), None, &HashMap::new()));
    }

    #[test]
    fn header_not_exists_matches_when_absent() {
        let rules = vec![rule("header", "X-Secret", "not_exists", None)];
        assert!(evaluate_rules(
            &rules,
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new()
        ));
    }

    #[test]
    fn query_contains_matches() {
        let rules = vec![rule("query", "role", "contains", Some("admin"))];
        let q = query(&[("role", "super_admin")]);
        assert!(evaluate_rules(&rules, &HashMap::new(), &q, None, &HashMap::new()));
    }

    #[test]
    fn body_eq_matches() {
        let rules = vec![rule("body", "status", "eq", Some("active"))];
        let body = serde_json::json!({"status": "active"});
        assert!(evaluate_rules(
            &rules,
            &HashMap::new(),
            &HashMap::new(),
            Some(&body),
            &HashMap::new()
        ));
    }

    #[test]
    fn multiple_rules_all_must_match() {
        let rules = vec![
            rule("header", "X-Role", "eq", Some("admin")),
            rule("query", "active", "eq", Some("true")),
        ];
        let h = headers(&[("X-Role", "admin")]);
        let q_pass = query(&[("active", "true")]);
        let q_fail = query(&[("active", "false")]);
        assert!(evaluate_rules(&rules, &h, &q_pass, None, &HashMap::new()));
        assert!(!evaluate_rules(&rules, &h, &q_fail, None, &HashMap::new()));
    }

    #[test]
    fn path_param_eq_matches() {
        let rules = vec![rule("path_param", "id", "eq", Some("42"))];
        let params: HashMap<String, String> =
            [("id".to_string(), "42".to_string())].into();
        assert!(evaluate_rules(
            &rules,
            &HashMap::new(),
            &HashMap::new(),
            None,
            &params
        ));
    }
}
