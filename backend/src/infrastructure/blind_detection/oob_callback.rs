use crate::domain::entities::*;
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

/// Out-of-band callback detection engine.
/// Generates unique per-scan callback identifiers and injects them into
/// target parameters to detect blind SSRF / XXE via DNS or HTTP interaction.
pub struct OobCallbackEngine {
    client: Client,
    /// Base domain used for OOB callbacks (configurable via env var).
    /// Should point to a real interact.sh instance or self-hosted listener.
    callback_base_domain: String,
}

impl Default for OobCallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OobCallbackEngine {
    pub fn new() -> Self {
        let callback_base_domain = std::env::var("LIMMA_OOB_DOMAIN")
            .unwrap_or_else(|_| "oob.limma-security.io".to_string());

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            callback_base_domain,
        }
    }

    /// Generate a unique callback URL for this scan interaction.
    /// Format: <uuid>.<base_domain> — each scan gets a unique subdomain
    /// so interactions can be correlated back to a specific injection point.
    fn generate_callback_url(&self) -> (String, String) {
        let callback_id = Uuid::new_v4().to_string();
        let callback_domain = format!("{}.{}", callback_id, self.callback_base_domain);
        let callback_url = format!("http://{}", callback_domain);
        (callback_id, callback_url)
    }

    /// Detect SSRF or XXE using an Out-Of-Band callback approach.
    ///
    /// 1. Generates a unique callback URL per scan
    /// 2. Injects it into target URL parameters (url, webhook, redirect, proxy, etc.)
    /// 3. Sends the request and waits for potential OOB interaction
    /// 4. Polls for received callbacks (DNS/HTTP hit on the unique subdomain)
    pub async fn detect_ssrf(&self, target_url: &str) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();
        tracing::debug!(
            "[OobCallbackEngine] Starting OOB analysis for {}",
            target_url
        );

        // Common SSRF injection parameter names
        let injection_params = ["url", "webhook", "redirect", "proxy", "callback", "fetch", "uri"];

        for param in &injection_params {
            let (callback_id, callback_url) = self.generate_callback_url();

            // Build test URL with injection
            let test_url = if target_url.contains('?') {
                format!("{}&{}={}", target_url, param, callback_url)
            } else {
                format!("{}?{}={}", target_url, param, callback_url)
            };

            tracing::debug!(
                "[OobCallbackEngine] Injecting OOB payload via param '{}': {}",
                param,
                test_url
            );

            // Send the injection request (fire-and-forget style)
            let send_result = self.client.get(&test_url).send().await;
            if let Err(e) = &send_result {
                tracing::debug!(
                    "[OobCallbackEngine] Request to {} failed (expected for some targets): {}",
                    test_url,
                    e
                );
            }

            // Wait for potential OOB callback to arrive
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Poll for callback on the unique subdomain.
            // In production, this queries your OOB listener's API (e.g. interact.sh polling endpoint)
            // to check if any DNS/HTTP interaction was received for callback_id.
            let interaction_received = self.poll_for_interaction(&callback_id).await;

            if interaction_received {
                tracing::info!(
                    "[OobCallbackEngine] OOB interaction received for callback_id={} via param '{}'",
                    callback_id,
                    param
                );

                findings.push(RawBlindFinding {
                    vulnerability_type: BlindVulnType::BlindSsrfHttp,
                    detection_method: BlindDetectionMethod::OobCallback {
                        callback_id: callback_id.clone(),
                    },
                    target_url: target_url.to_string(),
                    parameter: Some(param.to_string()),
                    payload_used: callback_url.clone(),
                    raw_confidence: 0.92,
                    evidence: BlindEvidence {
                        dom_snapshot: None,
                        timing_comparison: None,
                        callback_received: Some(CallbackData {
                            callback_id: callback_id.clone(),
                            received_at: chrono::Utc::now(),
                            source_ip: None,
                            response_data: Some(format!(
                                "OOB interaction detected on {}.{} via parameter '{}'",
                                callback_id, self.callback_base_domain, param
                            )),
                        }),
                        payload_hash: format!("{:x}", md5_hash(&callback_url)),
                    },
                });

                // One confirmed SSRF is enough — don't spam findings
                break;
            }
        }

        if findings.is_empty() {
            tracing::debug!(
                "[OobCallbackEngine] No OOB interactions detected for {}",
                target_url
            );
        }

        Ok(findings)
    }

    /// Poll the OOB listener API for any DNS/HTTP interaction matching the callback_id.
    ///
    /// In a production deployment, this would query your interact.sh server:
    ///   GET https://<interact_server>/poll?id=<callback_id>&secret=<api_secret>
    ///
    /// Returns true if an interaction was received.
    async fn poll_for_interaction(&self, callback_id: &str) -> bool {
        let poll_url = format!(
            "https://{}/api/poll?id={}",
            self.callback_base_domain, callback_id
        );

        match self.client.get(&poll_url).send().await {
            Ok(response) => {
                if let Ok(body) = response.text().await {
                    // interact.sh returns JSON with "data" array; non-empty = interaction received
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(data) = json.get("data") {
                            if let Some(arr) = data.as_array() {
                                return !arr.is_empty();
                            }
                        }
                        // Alternative: some servers return "aes_key" + non-null results
                        if json.get("aes_key").is_some() && json.get("data").is_some() {
                            return body.len() > 50; // heuristic: real data is larger
                        }
                    }
                }
                false
            }
            Err(e) => {
                tracing::debug!(
                    "[OobCallbackEngine] Failed to poll for interaction {}: {}",
                    callback_id,
                    e
                );
                false
            }
        }
    }
}

/// Simple hash for payload deduplication
fn md5_hash(input: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}
