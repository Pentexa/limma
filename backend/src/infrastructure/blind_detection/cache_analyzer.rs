use crate::domain::entities::*;
use reqwest::header::COOKIE;

pub struct CacheAnalyzer {
    client: reqwest::Client,
}

impl Default for CacheAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheAnalyzer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .cookie_store(true)
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn detect_cache_deception(
        &self,
        target_url: &str,
        session_cookie: Option<&str>,
    ) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();
        let session_cookie = session_cookie
            .map(str::trim)
            .filter(|cookie| !cookie.is_empty())
            .ok_or_else(|| {
                "Web cache deception detection requires the active scan session cookie".to_string()
            })?;

        let extensions = vec![".css", ".js", ".png", ".jpg", ".gif"];

        for ext in extensions {
            let mut test_url = target_url.trim_end_matches('/').to_string();
            test_url.push_str("/nonexistent_test");
            test_url.push_str(ext);

            // 1. Authenticated Request
            let auth_res = self
                .client
                .get(&test_url)
                .header(COOKIE, session_cookie)
                .send()
                .await;

            if let Ok(auth_response) = auth_res {
                let cache_status = auth_response
                    .headers()
                    .get("X-Cache")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("")
                    .to_uppercase();

                // Read body to see if it's dynamic content (HTML) instead of static
                if let Ok(auth_body) = auth_response.text().await {
                    if (cache_status.contains("HIT") || cache_status.contains("MISS"))
                        && auth_body.to_lowercase().contains("<html")
                    {
                        // 2. Unauthenticated Request
                        let unauth_res = self.client.get(&test_url).send().await;

                        if let Ok(unauth_response) = unauth_res {
                            if let Ok(unauth_body) = unauth_response.text().await {
                                // If unauthenticated user gets the exact same dynamic HTML body
                                // that the authenticated user got, cache deception is highly likely.
                                if unauth_body == auth_body && unauth_body.len() > 100 {
                                    findings.push(RawBlindFinding {
                                        target_url: test_url.clone(),
                                        parameter: Some("URL Path Extension".to_string()),
                                        vulnerability_type: BlindVulnType::WebCacheDeception,
                                        detection_method:
                                            BlindDetectionMethod::DifferentialAnalysis,
                                        payload_used: ext.to_string(),
                                        raw_confidence: 0.9,
                                        evidence: BlindEvidence {
                                            dom_snapshot: Some(
                                                unauth_body
                                                    [..std::cmp::min(500, unauth_body.len())]
                                                    .to_string(),
                                            ),
                                            timing_comparison: None,
                                            callback_received: None,
                                            payload_hash: format!("cache_deception_{}", test_url),
                                        },
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(findings)
    }
}
