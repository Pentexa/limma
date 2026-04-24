use crate::domain::entities::RuntimeVerification;
use reqwest::{Client, ClientBuilder};
use std::time::{Duration, Instant};

pub struct CrawlerFetcher {
    client: Client,
}

impl CrawlerFetcher {
    pub fn new() -> Self {
        // Build a powerful client with cookies, timeouts, compression, and redirect following.
        let client = ClientBuilder::new()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .cookie_store(true)
            .brotli(true)
            .gzip(true)
            .deflate(true)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    pub async fn fetch_html(&self, url: &str) -> Result<String, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // Ensure we are only parsing text-based formats
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html");

        if !content_type.contains("text/") && !content_type.contains("application/json") {
            return Err("Non-text content".to_string());
        }

        resp.text().await.map_err(|e| e.to_string())
    }

    pub async fn fetch_js(&self, url: &str) -> Result<String, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Prevent downloading massive PDFs or binary chunks masquerading as JS
        if content_type.contains("image")
            || content_type.contains("video")
            || content_type.contains("font")
        {
            return Err("Static asset detected via MIME".to_string());
        }

        resp.text().await.map_err(|e| e.to_string())
    }

    pub async fn test_endpoint(&self, url: &str) -> bool {
        // Quick HEAD/GET test for common endpoints
        if let Ok(resp) = self
            .client
            .get(url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            if resp.status().is_success() {
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                // If it's returning HTML it might be a catch-all soft-404 SPA fallback.
                return !content_type.contains("text/html");
            }
        }
        false
    }

    pub async fn verify_endpoint_deep(
        &self,
        url: &str,
        predicted_method: &str,
    ) -> Option<RuntimeVerification> {
        let attempts = match predicted_method {
            "POST" | "PUT" | "PATCH" | "DELETE" => vec!["OPTIONS", predicted_method, "GET"],
            "GET" => vec!["HEAD", "GET"],
            _ => vec!["HEAD", "GET", "POST"],
        };

        let mut best_result: Option<RuntimeVerification> = None;

        for method in attempts {
            let request = match method {
                "HEAD" => self.client.head(url),
                "GET" => self.client.get(url),
                "OPTIONS" => self.client.request(reqwest::Method::OPTIONS, url),
                "POST" => self.client.post(url),
                "PUT" => self.client.put(url),
                "PATCH" => self.client.patch(url),
                "DELETE" => self.client.delete(url),
                _ => self.client.get(url),
            };

            let start_time = Instant::now();
            if let Ok(resp) = request.timeout(Duration::from_millis(3500)).send().await {
                let duration = start_time.elapsed().as_millis() as u64;
                let status = resp.status();
                let status_u16 = status.as_u16();

                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let server = resp
                    .headers()
                    .get(reqwest::header::SERVER)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                // Ensure we fully finish the request and measure body if applicable
                let has_body = if method != "HEAD" && method != "OPTIONS" {
                    if let Ok(bytes) = resp.bytes().await {
                        !bytes.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                };

                let is_spa_fallback = status.is_success()
                    && method == "GET"
                    && content_type.as_deref().unwrap_or("").contains("text/html");

                let is_valid = if is_spa_fallback {
                    false
                } else {
                    status.is_success()
                        || status.is_redirection()
                        || status_u16 == 401
                        || status_u16 == 403
                        || status_u16 == 405
                        || status_u16 == 422
                };

                let current_verification = RuntimeVerification {
                    is_valid,
                    best_method: method.to_string(),
                    status_code: status_u16,
                    response_time_ms: duration,
                    has_body,
                    content_type,
                    server,
                };

                if is_valid {
                    return Some(current_verification);
                } else {
                    best_result = Some(current_verification);
                }
            }
        }

        best_result
    }
}
