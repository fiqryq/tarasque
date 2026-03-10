use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use uuid::Uuid;

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(
        body: &str,
        params: &HashMap<String, String>,
        query: &HashMap<String, String>,
        headers: &HashMap<String, String>,
        body_json: Option<&serde_json::Value>,
    ) -> String {
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
            let placeholder = format!("{{request.params.{}}}", key);
            result = result.replace(&placeholder, value);
        }

        for (key, value) in query {
            let placeholder = format!("{{request.query.{}}}", key);
            result = result.replace(&placeholder, value);
        }

        for (key, value) in headers {
            let placeholder = format!("{{request.headers.{}}}", key);
            result = result.replace(&placeholder, value);
        }

        if let Some(json) = body_json {
            Self::render_json_paths(&mut result, json, "request.body");
        }

        result
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
                let placeholder = format!("{{.{}.{}}}", prefix, key);
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

    #[test]
    fn test_uuid() {
        let result = TemplateEngine::render(
            "{{uuid}}",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        );
        assert_eq!(result.len(), 36);
    }

    #[test]
    fn test_timestamp() {
        let result = TemplateEngine::render(
            "{{timestamp}}",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        );
        assert!(result.parse::<i64>().is_ok());
    }

    #[test]
    fn test_request_params() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        let result = TemplateEngine::render(
            r#"{"user_id": "{{request.params.id}}"}"#,
            &params,
            &HashMap::new(),
            &HashMap::new(),
            None,
        );
        assert_eq!(result, r#"{"user_id": "123"}"#);
    }

    #[test]
    fn test_request_query() {
        let mut query = HashMap::new();
        query.insert("page".to_string(), "1".to_string());
        let result = TemplateEngine::render(
            r#"{"page": "{{request.query.page}}"}"#,
            &HashMap::new(),
            &query,
            &HashMap::new(),
            None,
        );
        assert_eq!(result, r#"{"page": "1"}"#);
    }

    #[test]
    fn test_request_body() {
        let body = serde_json::json!({"name": "John", "age": 30});
        let result = TemplateEngine::render(
            r#"{"name": "{{request.body.name}}", "age": {{request.body.age}}}"#,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            Some(&body),
        );
        assert_eq!(result, r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_random() {
        let result = TemplateEngine::render(
            "{{random.int}}",
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        );
        assert!(result.parse::<u32>().is_ok());
    }
}
