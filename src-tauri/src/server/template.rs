use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use uuid::Uuid;

pub enum StoreOp {
    Set(String, serde_json::Value),
    Delete(String),
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(
        body: &str,
        params: &HashMap<String, String>,
        query: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body_json: Option<&serde_json::Value>,
        store: &HashMap<String, serde_json::Value>,
        env_vars: &HashMap<String, String>,
    ) -> (String, Vec<StoreOp>) {
        let mut result = body.to_string();

        result = result.replace("{{uuid}}", &Uuid::new_v4().to_string());
        result = result.replace("{{timestamp}}", &Utc::now().timestamp().to_string());
        result = result.replace("{{iso_timestamp}}", &Utc::now().to_rfc3339());

        let mut rng = rand::thread_rng();
        result = result.replace("{{random.int}}", &rng.gen_range(0..1000).to_string());
        result = result.replace(
            "{{random.bool}}",
            if rng.gen_bool(0.5) { "true" } else { "false" },
        );
        result = result.replace("{{random.string}}", &Self::random_string(6));

        for (key, value) in params {
            let placeholder = format!("{{{{request.params.{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        for (key, value) in query {
            let placeholder = format!("{{{{request.query.{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        for (key, value) in headers {
            let placeholder = format!("{{{{request.headers.{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        if let Some(json) = body_json {
            Self::render_json_paths(&mut result, json, "request.body");
        }

        // Environment variable substitution: {{env.key}}
        for (key, value) in env_vars {
            let placeholder = format!("{{{{env.{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Store reads: {{store.key}}
        for (key, value) in store {
            let placeholder = format!("{{{{store.{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        // Store ops — collected and returned for the caller to apply
        let mut ops = Vec::new();

        // Write: {{store.set.key=value}}
        loop {
            if let Some(start) = result.find("{{store.set.") {
                if let Some(rel_end) = result[start..].find("}}") {
                    let inner = result[start + 12..start + rel_end].to_string();
                    if let Some(eq) = inner.find('=') {
                        let k = inner[..eq].to_string();
                        let v = serde_json::Value::String(inner[eq + 1..].to_string());
                        ops.push(StoreOp::Set(k, v));
                    }
                    result = format!("{}{}", &result[..start], &result[start + rel_end + 2..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Delete: {{store.delete.key}}
        loop {
            if let Some(start) = result.find("{{store.delete.") {
                if let Some(rel_end) = result[start..].find("}}") {
                    let key = result[start + 15..start + rel_end].to_string();
                    ops.push(StoreOp::Delete(key));
                    result = format!("{}{}", &result[..start], &result[start + rel_end + 2..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        (result, ops)
    }

    fn random_string(len: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn render_json_paths(result: &mut String, json: &serde_json::Value, prefix: &str) {
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}.{}}}}}", prefix, key);
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                *result = result.replace(&placeholder, &value_str);

                if let serde_json::Value::Object(_) = value {
                    Self::render_json_paths(result, value, &format!("{}.{}", prefix, key));
                }
            }
        } else if let Some(arr) = json.as_array() {
            for (idx, item) in arr.iter().enumerate() {
                let placeholder = format!("{{.{}.{}}}", prefix, idx);
                let value_str = match item {
                    serde_json::Value::String(s) => s.clone(),
                    _ => item.to_string(),
                };
                *result = result.replace(&placeholder, &value_str);

                Self::render_json_paths(result, item, &format!("{}[{}]", prefix, idx));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(body: &str) -> String {
        TemplateEngine::render(body, &HashMap::new(), &HashMap::new(), &HashMap::new(), None, &HashMap::new(), &HashMap::new()).0
    }

    #[test]
    fn test_uuid() {
        assert_eq!(render("{{uuid}}").len(), 36);
    }

    #[test]
    fn test_timestamp() {
        assert!(render("{{timestamp}}").parse::<i64>().is_ok());
    }

    #[test]
    fn test_request_params() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        let (result, _) = TemplateEngine::render(
            r#"{"user_id": "{{request.params.id}}"}"#,
            &params,
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"user_id": "123"}"#);
    }

    #[test]
    fn test_request_query() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        let (result, _) = TemplateEngine::render(
            r#"{"page": "{{request.query.page}}"}"#,
            &HashMap::new(),
            &query,
            &HashMap::new(),
            None,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"page": "1"}"#);
    }

    #[test]
    fn test_request_body() {
        let body = serde_json::json!({"name": "John", "age": 30});
        let (result, _) = TemplateEngine::render(
            r#"{"name": "{{request.body.name}}", "age": {{request.body.age}}}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            Some(&body),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_random() {
        assert!(render("{{random.int}}").parse::<u32>().is_ok());
    }

    #[test]
    fn store_read_substitutes_value() {
        let store: HashMap<String, serde_json::Value> = [(
            "token".to_string(),
            serde_json::Value::String("abc123".to_string()),
        )]
        .into();
        let (result, _) = TemplateEngine::render(
            r#"{"token": "{{store.token}}"}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
            &store,
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"token": "abc123"}"#);
    }

    #[test]
    fn store_set_returns_op_and_removes_placeholder() {
        let (result, ops) = TemplateEngine::render(
            r#"{"ok": true}{{store.set.lastId=42}}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"ok": true}"#);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            StoreOp::Set(k, _) => assert_eq!(k, "lastId"),
            _ => panic!("Expected Set op"),
        }
    }

    #[test]
    fn test_env_variable() {
        let mut env_vars = HashMap::new();
        env_vars.insert("baseUrl".to_string(), "https://example.com".to_string());
        let (result, _) = TemplateEngine::render(
            r#"{"url": "{{env.baseUrl}}"}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new(),
            &env_vars,
        );
        assert_eq!(result, r#"{"url": "https://example.com"}"#);
    }

    #[test]
    fn store_delete_returns_op_and_removes_placeholder() {
        let (result, ops) = TemplateEngine::render(
            r#"{"ok": true}{{store.delete.token}}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(result, r#"{"ok": true}"#);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            StoreOp::Delete(k) => assert_eq!(k, "token"),
            _ => panic!("Expected Delete op"),
        }
    }
}
