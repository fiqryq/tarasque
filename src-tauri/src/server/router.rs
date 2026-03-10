use std::collections::HashMap;

pub fn match_path(pattern: &str, actual: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let actual_parts: Vec<&str> = actual.split('/').collect();

    let mut params = HashMap::new();
    let mut pi = 0;
    let mut ai = 0;

    while pi < pattern_parts.len() {
        let p = pattern_parts[pi];

        if p == "**" {
            // Match any remaining segments (greedy)
            return Some(params);
        } else if p == "*" {
            // Match exactly one segment
            if ai >= actual_parts.len() {
                return None;
            }
            ai += 1;
            pi += 1;
        } else if let Some(key) = p.strip_prefix(':') {
            if ai >= actual_parts.len() {
                return None;
            }
            params.insert(key.to_string(), actual_parts[ai].to_string());
            ai += 1;
            pi += 1;
        } else {
            if ai >= actual_parts.len() || p != actual_parts[ai] {
                return None;
            }
            ai += 1;
            pi += 1;
        }
    }

    if ai != actual_parts.len() {
        return None;
    }

    Some(params)
}

pub fn substitute_params(body: &str, params: &HashMap<String, String>) -> String {
    let mut result = body.to_string();
    for (key, value) in params {
        result = result.replace(&format!(":{key}"), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_returns_empty_params() {
        let result = match_path("/users", "/users");
        assert_eq!(result, Some(HashMap::new()));
    }

    #[test]
    fn exact_mismatch_returns_none() {
        let result = match_path("/users", "/posts");
        assert_eq!(result, None);
    }

    #[test]
    fn param_match_extracts_value() {
        let result = match_path("/users/:id", "/users/123");
        let mut expected = HashMap::new();
        expected.insert("id".to_string(), "123".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn multiple_params_extracted() {
        let result = match_path("/users/:userId/posts/:postId", "/users/42/posts/7");
        let mut expected = HashMap::new();
        expected.insert("userId".to_string(), "42".to_string());
        expected.insert("postId".to_string(), "7".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn different_segment_count_returns_none() {
        let result = match_path("/users/:id", "/users/123/extra");
        assert_eq!(result, None);
    }

    #[test]
    fn substitute_replaces_param_in_body() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        let result = substitute_params(r#"{"id": ":id"}"#, &params);
        assert_eq!(result, r#"{"id": "123"}"#);
    }

    #[test]
    fn substitute_no_params_returns_original() {
        let params = HashMap::new();
        let body = r#"{"message": "hello"}"#;
        assert_eq!(substitute_params(body, &params), body);
    }

    #[test]
    fn wildcard_single_matches_one_segment() {
        let result = match_path("/api/*", "/api/users");
        assert!(result.is_some());
    }

    #[test]
    fn wildcard_single_does_not_match_two_segments() {
        let result = match_path("/api/*", "/api/users/123");
        assert!(result.is_none());
    }

    #[test]
    fn wildcard_double_matches_one_segment() {
        let result = match_path("/api/**", "/api/users");
        assert!(result.is_some());
    }

    #[test]
    fn wildcard_double_matches_multiple_segments() {
        let result = match_path("/api/**", "/api/users/123/posts");
        assert!(result.is_some());
    }

    #[test]
    fn wildcard_double_matches_empty_tail() {
        let result = match_path("/api/**", "/api/");
        assert!(result.is_some());
    }
}
