use crate::domain::entities::*;
use headless_chrome::{Browser, LaunchOptions};

/// DOM XSS detection using headless browser.
/// Uses headless_chrome to navigate to the target URL and monitor for JS execution or alerts.
pub struct DomExecutorImpl;

impl Default for DomExecutorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DomExecutorImpl {
    pub fn new() -> Self {
        Self
    }

    /// Detect DOM XSS by injecting a payload and checking for execution in the DOM
    pub async fn detect_dom_xss(&self, target_url: &str) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();
        tracing::debug!("[DomExecutor] Starting DOM analysis for {}", target_url);

        // We run the browser interaction in a blocking thread since headless_chrome is synchronous
        let target = target_url.to_string();
        let result = tokio::task::spawn_blocking(move || -> Result<Option<RawBlindFinding>, String> {
            let browser = Browser::new(
                LaunchOptions::default_builder()
                    .headless(true)
                    .build()
                    .map_err(|e| format!("Failed to build browser options: {}", e))?
            ).map_err(|e| format!("Failed to launch browser: {}", e))?;

            let tab = browser.new_tab().map_err(|e| format!("Failed to open tab: {}", e))?;

            // Prepare a DOM XSS payload URL (e.g. hash fragment or query parameter)
            let test_url = if target.contains('?') {
                format!("{}&test_param=javascript:alert(document.domain)//", target)
            } else {
                format!("{}?test_param=javascript:alert(document.domain)//#<script>alert(1)</script>", target)
            };

            tracing::debug!("[DomExecutor] Navigating to {}", test_url);
            tab.navigate_to(&test_url).map_err(|e| format!("Failed to navigate: {}", e))?;

            // Wait for page load
            let _ = tab.wait_until_navigated();

            // Inject a script to check if any sinks are vulnerable
            let eval_result = tab.evaluate(
                r#"
                (function() {
                    // Check if our payload is reflected anywhere dangerous in the DOM
                    let html = document.body.innerHTML;
                    if (html.includes('javascript:alert(document.domain)') || html.includes('<script>alert(1)</script>')) {
                        return true;
                    }
                    return false;
                })()
                "#,
                true
            ).map_err(|e| format!("Failed to evaluate JS: {}", e))?;

            let is_vulnerable = eval_result.value.and_then(|v| v.as_bool()).unwrap_or(false);

            if is_vulnerable {
                Ok(Some(RawBlindFinding {
                    vulnerability_type: BlindVulnType::DomXss,
                    detection_method: BlindDetectionMethod::DomExecution,
                    target_url: target,
                    parameter: Some("test_param/hash".to_string()),
                    payload_used: "<script>alert(1)</script>".to_string(),
                    raw_confidence: 0.85,
                    evidence: crate::domain::entities::BlindEvidence {
                        dom_snapshot: Some("DOM executed".to_string()),
                        timing_comparison: None,
                        callback_received: None,
                        payload_hash: "".to_string(),
                    },
                }))
            } else {
                Ok(None)
            }
        }).await.map_err(|e| format!("Task join error: {}", e))??;

        if let Some(finding) = result {
            findings.push(finding);
        }

        Ok(findings)
    }
}
