use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::sync::{Arc, Mutex};

pub struct XssDetector {
    client: Client,
}

impl XssDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn check_xss_reflection(&self, body: &str, payload: &str) -> (bool, bool) {
        if !body.contains(payload) {
            return (false, false);
        }

        let encoded = payload.replace('<', "&lt;").replace('>', "&gt;");
        if body.contains(&encoded) && !body.contains(payload) {
            return (false, false);
        }

        let dangerous_indicators = [
            "<script",
            "onerror=",
            "onload=",
            "onclick=",
            "onfocus=",
            "onmouseover=",
            "ontoggle=",
            "javascript:",
        ];

        let is_dangerous = dangerous_indicators.iter().any(|ind| {
            payload.to_lowercase().contains(&ind.to_lowercase())
                || body.to_lowercase().contains(&ind.to_lowercase())
        });

        (true, is_dangerous)
    }

    fn headless_verify(&self, test_url: &str) -> bool {
        let options = LaunchOptionsBuilder::default()
            .headless(true)
            .idle_browser_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        if let Ok(browser) = Browser::new(options) {
            if let Ok(tab) = browser.new_tab() {
                let triggered = Arc::new(Mutex::new(false));
                let triggered_clone = triggered.clone();

                let _ = tab.add_event_listener(Arc::new(move |event: &headless_chrome::protocol::cdp::types::Event| {
                    if let headless_chrome::protocol::cdp::types::Event::PageJavascriptDialogOpening(ref _dialog) = event {
                        if let Ok(mut lock) = triggered_clone.lock() {
                            *lock = true;
                        }
                    }
                }));

                // Attempt navigation
                let _ = tab.navigate_to(test_url);
                std::thread::sleep(std::time::Duration::from_secs(3));

                let res = *triggered.lock().unwrap();
                return res;
            }
        }
        false
    }
}

#[async_trait]
impl VulnDetector for XssDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![
            ActiveVulnType::ReflectedXss,
            ActiveVulnType::StoredXss,
            ActiveVulnType::DomXss,
        ]
    }

    async fn detect(
        &self,
        target_url: &str,
        parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>,
        baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::ReflectedXss);

        for payload_def in &payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }

            let payload_response = super::send_payload_request(
                &self.client,
                target_url,
                parameter,
                &payload_def.payload,
                endpoint_ctx,
                insertion_point,
                payload_selector.is_waf_bypass_enabled(),
            )
            .await?;

            waf_monitor
                .register_response(&payload_response.request_url, payload_response.status_code);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let body = payload_response.response_body.clone();

            let (reflected, dangerous) = self.check_xss_reflection(&body, &payload_def.payload);
            if reflected {
                if !dangerous {
                    // Safe reflection (HTML encoded). Not a vulnerability.
                    continue;
                }

                let is_false_positive = baseline
                    .map(|b| b.contains_indicator(&payload_def.payload))
                    .unwrap_or(false);

                if is_false_positive {
                    continue;
                }

                let mut finding_severity = SeverityLevel::Medium;
                let mut finding_confidence = ConfidenceLevel::Firm;
                let mut additional_notes = vec![
                    payload_def.description.clone(),
                    "Payload reflected without HTML encoding".to_string(),
                    "Reflection is in a potentially dangerous context (Potential XSS)".to_string(),
                ];

                // Headless Verification is only possible for browser-navigable GET query payloads.
                if payload_response
                    .browser_verification_url
                    .as_deref()
                    .is_some_and(|test_url| self.headless_verify(test_url))
                {
                    finding_severity = SeverityLevel::High;
                    finding_confidence = ConfidenceLevel::Certain;
                    additional_notes
                        .push("Headless Verification: Javascript Execution Confirmed!".to_string());
                } else {
                    additional_notes.push("Headless Verification: Not confirmed or not applicable for this insertion point".to_string());
                    // Optional: skip adding the finding if we strictly want only confirmed
                    // continue;
                }

                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::ReflectedXss,
                    target_url: payload_response.request_url.clone(),
                    affected_parameter: parameter.to_string(),
                    http_method: payload_response.http_method.clone(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: payload_response.request_raw,
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: payload_response.response_time_ms,
                        matched_indicator: format!("XSS payload reflected: {}", payload_def.id),
                        additional_notes,
                    },
                    severity: finding_severity,
                    confidence: finding_confidence.clone(),
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: finding_confidence == ConfidenceLevel::Certain,
                    false_positive: false,
                });

                if finding_confidence == ConfidenceLevel::Certain {
                    break;
                }
            }
        }

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[test]
    fn test_xss_reflection_logic() {
        let detector = XssDetector::new(Client::new());
        let payload = "<script>alert(1)</script>";

        // 1. Valid reflection, dangerous
        let body = "Hello <script>alert(1)</script> World";
        let (reflected, dangerous) = detector.check_xss_reflection(body, payload);
        assert!(reflected);
        assert!(dangerous);

        // 2. HTML Encoded, should NOT be reflected
        let body = "Hello &lt;script&gt;alert(1)&lt;/script&gt; World";
        let (reflected, _dangerous) = detector.check_xss_reflection(body, payload);
        assert!(!reflected);

        // 3. Not reflected
        let body = "Hello World";
        let (reflected, _dangerous) = detector.check_xss_reflection(body, payload);
        assert!(!reflected);

        // 4. Reflected but not dangerous
        let payload_safe = "JohnDoe";
        let body_safe = "Hello JohnDoe";
        let (reflected, dangerous) = detector.check_xss_reflection(body_safe, payload_safe);
        assert!(reflected);
        assert!(!dangerous);
    }
}
