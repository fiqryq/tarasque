use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub status: u16,
    pub delay_ms: u64,
    pub headers: HashMap<String, String>,
    pub body: String,
    #[serde(default)]
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub method: String,
    pub path: String,
    pub scenarios: Vec<Scenario>,
    pub active_scenario_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub endpoints: Vec<Endpoint>,
    // Proxy forwarding
    pub proxy_url: Option<String>,
    pub proxy_enabled: bool,
    #[serde(default)]
    pub proxy_insecure: bool,   // accept self-signed / invalid TLS certs on upstream
    // Local HTTPS alias (e.g., https://api.local:8443)
    #[serde(default)]
    pub alias_enabled: bool,
    pub alias_hostname: Option<String>,
    pub alias_port: Option<u16>,
    pub alias_cert_pem: Option<String>,
    pub alias_key_pem: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppData {
    pub collections: Vec<Collection>,
    pub active_collection_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration: u64,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub response_body: Option<String>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestAssertion {
    pub id: String,
    #[serde(rename = "type")]
    pub assertion_type: String,
    pub path: Option<String>,
    pub operator: String,
    pub expected: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub method: String,
    pub path: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub assertions: Vec<TestAssertion>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestSuite {
    pub id: String,
    pub name: String,
    pub collection_id: String,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub test_case_id: String,
    pub passed: bool,
    pub duration: u64,
    pub actual_status: Option<u16>,
    pub actual_body: Option<String>,
    pub error: Option<String>,
    pub assertion_results: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestRunResult {
    pub suite_id: String,
    pub timestamp: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<TestResult>,
}
