use crate::models::{AppData, Collection, Endpoint, Scenario};
use crate::server::state::MultiServerHandle;
use crate::{server, storage};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn load_data(app: AppHandle) -> Result<AppData, String> {
    storage::load(&app).await
}

#[tauri::command]
pub async fn save_data(app: AppHandle, data: AppData) -> Result<(), String> {
    storage::save(&app, &data).await
}

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    multi: State<'_, MultiServerHandle>,
    collection_id: String,
    data: AppData,
) -> Result<(), String> {
    storage::save(&app, &data).await?;

    let collection = data
        .collections
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or_else(|| format!("Collection '{collection_id}' not found"))?;

    server::start(collection, app.clone(), &multi).await
}

#[tauri::command]
pub async fn stop_server(
    multi: State<'_, MultiServerHandle>,
    collection_id: String,
) -> Result<(), String> {
    server::stop(&collection_id, &multi).await
}

#[tauri::command]
pub async fn get_running_collections(
    multi: State<'_, MultiServerHandle>,
) -> Result<Vec<String>, String> {
    Ok(server::running_ids(&multi).await)
}

#[tauri::command]
pub fn find_free_port() -> Result<u16, String> {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find free port: {e}"))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Failed to read port: {e}"))?
        .port();
    Ok(port)
}

#[tauri::command]
pub fn import_openapi(content: String) -> Result<Vec<Collection>, String> {
    let spec: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let title = spec
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Imported")
        .to_string();

    let mut collection = Collection {
        id: uuid::Uuid::new_v4().to_string(),
        name: title,
        port: 3000,
        endpoints: Vec::new(),
        proxy_enabled: false,
        proxy_url: None,
        proxy_insecure: false,
        alias_enabled: false,
        alias_hostname: None,
        alias_port: None,
        alias_cert_pem: None,
        alias_key_pem: None,
        environments: Vec::new(),
        active_env_id: None,
    };

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, details) in methods_obj {
                    if !["get", "post", "put", "delete", "patch"].contains(&method.as_str()) {
                        continue;
                    }

                    if let Some(details_obj) = details.as_object() {
                        let responses = details_obj
                            .get("responses")
                            .and_then(|r| r.as_object());

                        let mut scenarios = Vec::new();

                        if let Some(resp) = responses {
                            for (status, response) in resp {
                                let status_code: u16 = status
                                    .parse()
                                    .unwrap_or(200);

                                let body = response
                                    .as_object()
                                    .and_then(|r| r.get("content"))
                                    .and_then(|c| c.get("application/json"))
                                    .and_then(|j| j.get("schema"))
                                    .map(|s| serde_json::to_string_pretty(&generate_example_schema(s)).unwrap_or_default())
                                    .unwrap_or_else(|| "{}".to_string());

                                let mut headers = HashMap::new();
                                headers.insert("Content-Type".to_string(), "application/json".to_string());

                                scenarios.push(Scenario {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: format!("{} {}", method.to_uppercase(), status),
                                    status: status_code,
                                    delay_ms: 0,
                                    headers,
                                    body,
                                    query_params: HashMap::new(),
                                    match_rules: Vec::new(),
                                });
                            }
                        }

                        if scenarios.is_empty() {
                            scenarios.push(Scenario {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: "Default".to_string(),
                                status: 200,
                                delay_ms: 0,
                                headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
                                body: "{}".to_string(),
                                query_params: HashMap::new(),
                                match_rules: Vec::new(),
                            });
                        }

                        let active_id = scenarios.first().map(|s| s.id.clone()).unwrap_or_default();

                        collection.endpoints.push(Endpoint {
                            id: uuid::Uuid::new_v4().to_string(),
                            method: method.to_uppercase(),
                            path: path.clone(),
                            scenarios,
                            active_scenario_id: active_id,
                            description: None,
                            sequence_mode: false,
                        });
                    }
                }
            }
        }
    }

    if collection.endpoints.is_empty() {
        return Err("No endpoints found in OpenAPI spec".to_string());
    }

    Ok(vec![collection])
}

fn generate_example_schema(schema: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = schema.get("properties").and_then(|p| p.as_object()) {
        let mut result = serde_json::Map::new();
        for (key, value) in obj {
            result.insert(key.clone(), generate_example_schema(value));
        }
        serde_json::Value::Object(result)
    } else if schema.get("type").and_then(|t| t.as_str()) == Some("array") {
        serde_json::Value::Array(vec![generate_example_schema(
            schema.get("items").unwrap_or(&serde_json::Value::Null),
        )])
    } else if schema.get("type").and_then(|t| t.as_str()) == Some("string") {
        serde_json::Value::String("string".to_string())
    } else if schema.get("type").and_then(|t| t.as_str()) == Some("integer") {
        serde_json::Value::Number(0.into())
    } else if schema.get("type").and_then(|t| t.as_str()) == Some("boolean") {
        serde_json::Value::Bool(false)
    } else {
        serde_json::Value::Null
    }
}

#[tauri::command]
pub fn export_collection_json(collection: Collection) -> Result<String, String> {
    serde_json::to_string_pretty(&collection)
        .map_err(|e| format!("Failed to serialize: {}", e))
}

#[tauri::command]
pub fn export_collection_postman(collection: Collection) -> Result<String, String> {
    let mut items = Vec::new();

    for ep in &collection.endpoints {
        let first_scenario = ep.scenarios.first();
        
        let mut req = serde_json::Map::new();
        req.insert("method".to_string(), serde_json::Value::String(ep.method.clone()));
        req.insert("header".to_string(), serde_json::Value::Array(Vec::new()));
        
        let mut url_obj = serde_json::Map::new();
        url_obj.insert("raw".to_string(), serde_json::Value::String(ep.path.clone()));
        url_obj.insert("host".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("{{baseUrl}}".to_string())]));
        url_obj.insert("path".to_string(), serde_json::Value::Array(
            ep.path.split('/').filter(|s| !s.is_empty())
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect()
        ));
        
        req.insert("url".to_string(), serde_json::Value::Object(url_obj));
        
        if let Some(scenario) = first_scenario {
            let mut body_obj = serde_json::Map::new();
            body_obj.insert("mode".to_string(), serde_json::Value::String("raw".to_string()));
            body_obj.insert("raw".to_string(), serde_json::Value::String(scenario.body.clone()));
            req.insert("body".to_string(), serde_json::Value::Object(body_obj));
        }

        let mut item = serde_json::Map::new();
        item.insert("name".to_string(), serde_json::Value::String(ep.path.clone()));
        item.insert("request".to_string(), serde_json::Value::Object(req));
        item.insert("response".to_string(), serde_json::Value::Array(
            ep.scenarios.iter().map(|s| {
                serde_json::json!({
                    "name": s.name.clone()
                })
            }).collect()
        ));
        
        items.push(serde_json::Value::Object(item));
    }

    let postman = serde_json::json!({
        "info": {
            "name": collection.name,
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "item": items
    });

    serde_json::to_string_pretty(&postman)
        .map_err(|e| format!("Failed to serialize: {}", e))
}

#[tauri::command]
pub async fn run_test_suite(
    suite: crate::models::TestSuite,
    base_url: String,
) -> Result<crate::models::TestRunResult, String> {
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for test_case in &suite.test_cases {
        let start = std::time::Instant::now();
        
        let url = format!("{}{}", base_url.trim_end_matches('/'), test_case.path);
        
        let mut request = client.request(
            reqwest::Method::from_bytes(test_case.method.as_bytes()).unwrap_or(reqwest::Method::GET),
            &url,
        );

        if let Some(headers) = &test_case.headers {
            for (k, v) in headers {
                request = request.header(k.as_str(), v.as_str());
            }
        }

        if let Some(body) = &test_case.body {
            request = request.body(body.clone());
        }

        let _assertion_results = match request.send().await {
            Ok(response) => {
                let duration = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                
                let mut case_passed = true;
                let mut case_results = Vec::new();

                for assertion in &test_case.assertions {
                    let (assertion_passed, actual) = match assertion.assertion_type.as_str() {
                        "status" => {
                            let expected = assertion.expected.as_u64().unwrap_or(200) as u16;
                            let passed = match assertion.operator.as_str() {
                                "eq" | _ => status == expected,
                            };
                            (passed, status.to_string())
                        }
                        "body" => {
                            let actual_val = if let Some(path) = &assertion.path {
                                extract_json_path(&body, path)
                            } else {
                                body.clone()
                            };
                            let expected_str = assertion.expected
                                .as_str()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| assertion.expected.to_string());
                            let passed = match assertion.operator.as_str() {
                                "eq"         => actual_val == expected_str,
                                "contains"   => actual_val.contains(&expected_str),
                                "exists"     => !actual_val.is_empty(),
                                "not_exists" => actual_val.is_empty(),
                                _            => false,
                            };
                            (passed, actual_val)
                        }
                        _ => (true, String::new()),
                    };
                    
                    case_passed &= assertion_passed;
                    case_results.push(serde_json::json!({
                        "assertion": assertion,
                        "passed": assertion_passed,
                        "actual": actual
                    }));
                }

                if case_passed {
                    passed += 1;
                } else {
                    failed += 1;
                }

                results.push(crate::models::TestResult {
                    test_case_id: test_case.id.clone(),
                    passed: case_passed,
                    duration,
                    actual_status: Some(status),
                    actual_body: Some(body),
                    error: None,
                    assertion_results: case_results,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(crate::models::TestResult {
                    test_case_id: test_case.id.clone(),
                    passed: false,
                    duration: start.elapsed().as_millis() as u64,
                    actual_status: None,
                    actual_body: None,
                    error: Some(e.to_string()),
                    assertion_results: vec![],
                });
            }
        };
    }

    Ok(crate::models::TestRunResult {
        suite_id: suite.id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        total: suite.test_cases.len(),
        passed,
        failed,
        results,
    })
}

#[derive(Serialize)]
pub struct AliasCredentials {
    pub cert_pem: String,
    pub key_pem: String,
}

/// Generate a self-signed TLS certificate for the given hostname.
/// Returns the PEM-encoded certificate and private key to be stored
/// in the collection and used by the HTTPS alias listener.
#[tauri::command]
pub fn generate_alias_cert(app: AppHandle, hostname: String) -> Result<AliasCredentials, String> {
    let san = vec![hostname.clone()];
    let certified = rcgen::generate_simple_self_signed(san)
        .map_err(|e| format!("Certificate generation failed: {e}"))?;

    let cert_pem = certified.cert.pem();
    let key_pem = certified.key_pair.serialize_pem();

    // Write to disk for Docker volume mount
    crate::proxy::write_cert_files(&app, &hostname, &cert_pem, &key_pem)?;

    Ok(AliasCredentials { cert_pem, key_pem })
}

/// Add `127.0.0.1 <hostname>` to /etc/hosts (requires admin password via macOS auth dialog).
#[tauri::command]
pub fn add_hosts_entry(hostname: String) -> Result<(), String> {
    crate::proxy::validate_hostname(&hostname)?;
    #[cfg(target_os = "macos")]
    {
        let entry = format!("127.0.0.1 {}", hostname);

        // Check if already present — no root needed for reading
        let hosts = std::fs::read_to_string("/etc/hosts").unwrap_or_default();
        if hosts.lines().any(|l| l.split_whitespace().collect::<Vec<_>>() == entry.split_whitespace().collect::<Vec<_>>()) {
            return Ok(());
        }

        let script = format!(
            "do shell script \"echo '{}' >> /etc/hosts\" with administrator privileges",
            entry
        );
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to run osascript: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to update /etc/hosts: {}", stderr))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = hostname;
        Err("Auto /etc/hosts is only supported on macOS".to_string())
    }
}

/// Trust the certificate in the macOS login Keychain (no admin required).
/// Falls back to opening the cert in Keychain Access if the direct command fails.
/// macOS only — returns an error on other platforms.
#[tauri::command]
pub fn trust_cert_macos(cert_pem: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;

        // Write cert to a temp file
        let temp_path = std::env::temp_dir().join("tarasque_alias_cert.crt");
        {
            let mut f = std::fs::File::create(&temp_path)
                .map_err(|e| format!("Failed to create temp file: {e}"))?;
            f.write_all(cert_pem.as_bytes())
                .map_err(|e| format!("Failed to write cert: {e}"))?;
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
        let keychain = format!("{}/Library/Keychains/login.keychain-db", home);
        let cert_path = temp_path.to_str().unwrap_or("/tmp/tarasque_alias_cert.crt").to_string();

        // Exact same approach as Ophiuchi desktop (open source, works in production).
        // Key: NO -r flag, NO -d flag, NO sudo, NO osascript.
        // The -r trustRoot flag triggers SecTrustSettingsSetTrustSettings which
        // requires Security framework authorization and blocks programmatic access.
        // Without -r, security adds the cert to the login keychain directly.
        let output = std::process::Command::new("security")
            .args(["add-trusted-cert", "-k", &keychain, &cert_path])
            .output()
            .map_err(|e| format!("Failed to run security: {e}"))?;

        let _ = std::fs::remove_file(&temp_path);

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!(
                "Failed to trust certificate: {}",
                if !stderr.is_empty() { stderr } else { stdout }
            ))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = cert_pem;
        Err("Auto-trust is only supported on macOS".to_string())
    }
}

#[tauri::command]
pub fn write_cert_files(
    app: AppHandle,
    hostname: String,
    cert_pem: String,
    key_pem: String,
) -> Result<(), String> {
    crate::proxy::write_cert_files(&app, &hostname, &cert_pem, &key_pem)
}

#[derive(serde::Deserialize)]
pub struct AliasEntry {
    pub hostname: String,
    pub port: u16,
}

#[tauri::command]
pub fn start_proxy(app: AppHandle, aliases: Vec<AliasEntry>) -> Result<(), String> {
    let pairs: Vec<(String, u16)> = aliases.into_iter().map(|a| (a.hostname, a.port)).collect();
    crate::proxy::start_proxy(&app, &pairs)
}

#[tauri::command]
pub fn stop_proxy(app: AppHandle) -> Result<(), String> {
    crate::proxy::stop_proxy(&app)
}

#[tauri::command]
pub fn get_proxy_status(app: AppHandle) -> Result<String, String> {
    Ok(crate::proxy::get_proxy_status(&app))
}

fn extract_json_path(body: &str, path: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &json;
        
        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(val) = obj.get(part) {
                    current = val;
                } else {
                    return String::new();
                }
            } else if let Some(arr) = current.as_array() {
                if let Ok(idx) = part.parse::<usize>() {
                    if let Some(val) = arr.get(idx) {
                        current = val;
                    } else {
                        return String::new();
                    }
                } else {
                    return String::new();
                }
            } else {
                return String::new();
            }
        }
        
        return current.to_string();
    }
    String::new()
}
