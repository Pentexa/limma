use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AuthType {
    #[default]
    None,
    Cookie(String),
    BearerToken(String),
    CustomHeaders(HashMap<String, String>),
    BurpRequest(String), // Raw HTTP request to extract headers/cookies from
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthProfile {
    pub id: String,
    pub name: String,
    pub auth_type: AuthType,
}

impl AuthProfile {
    /// Extracts a HashMap of headers to apply to requests based on this auth profile
    pub fn get_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        match &self.auth_type {
            AuthType::None => {}
            AuthType::Cookie(cookie) => {
                headers.insert("Cookie".to_string(), cookie.clone());
            }
            AuthType::BearerToken(token) => {
                headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            }
            AuthType::CustomHeaders(custom) => {
                for (k, v) in custom {
                    headers.insert(k.clone(), v.clone());
                }
            }
            AuthType::BurpRequest(raw_req) => {
                // Parse basic headers from raw Burp request
                // Very rudimentary parser: split by \r\n, find Key: Value
                let lines: Vec<&str> = raw_req.split("\n").collect();
                for line in lines {
                    let line = line.trim();
                    if line.is_empty() {
                        break; // End of headers
                    }
                    if let Some(idx) = line.find(':') {
                        let key = line[..idx].trim().to_string();
                        let value = line[idx + 1..].trim().to_string();
                        // Ignore Host, Content-Length etc that might conflict, mostly we want Cookie, Auth
                        let lower_key = key.to_lowercase();
                        if lower_key == "cookie"
                            || lower_key == "authorization"
                            || lower_key.starts_with("x-")
                        {
                            headers.insert(key, value);
                        }
                    }
                }
            }
        }
        headers
    }
}
