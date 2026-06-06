use reqwest::Method;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsertionPoint {
    QueryParam(String),
    Header(String),
    JsonBodyPath(String), // JSON pointer or field path
    FormData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointContext {
    pub method: String,
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>, // Can be JSON string, raw data, etc.
}

impl EndpointContext {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_uppercase(),
            url: url.to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub endpoint: EndpointContext,
    pub insertion_points: Vec<InsertionPoint>,
}
