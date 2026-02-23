use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub params_raw: String,
    pub headers_raw: String,
    pub body: String,
    #[serde(default = "default_body_format")]
    pub body_format: String,
    #[serde(default)]
    pub custom_content_type: String,
}

impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            name: "Untitled Request".into(),
            method: "GET".into(),
            url: String::new(),
            params_raw: String::new(),
            headers_raw: String::new(),
            body: String::new(),
            body_format: default_body_format(),
            custom_content_type: String::new(),
        }
    }
}

fn default_body_format() -> String {
    "none".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status_code: u16,
    pub reason: String,
    pub elapsed_ms: u128,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub request: ApiRequest,
    pub response: ApiResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub collections: Vec<ApiRequest>,
    pub history: Vec<HistoryEntry>,
    pub env_raw: String,
}
