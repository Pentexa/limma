use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};
use crate::infrastructure::active_detection::evidence::reflection_analyzer::ReflectionAnalyzer;
use crate::infrastructure::active_detection::evidence::EvidenceStrength;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::sync::{Arc, Mutex};

pub struct XssDetector {
    client: Client,
}

impl XssDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    #[cfg(test)]
    fn check_xss_reflection(&self, body: &str, payload: &str) -> (bool, bool) {
        let analysis = ReflectionAnalyzer::analyze(body, payload);
        (analysis.reflected, analysis.dangerous_context)
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

            let analysis = ReflectionAnalyzer::analyze(&body, &payload_def.payload);
            if !analysis.reflected || !analysis.dangerous_context {
                continue;
            }

            let Some(mut evidence) = analysis.evidence else {
                continue;
            };

            let mut finding_severity = SeverityLevel::Medium;
            evidence.summary = format!(
                "XSS payload reflected without HTML encoding in dangerous context ({})",
                payload_def.description
            );

            // Headless verification is only possible for browser-navigable GET query payloads.
            if payload_response
                .browser_verification_url
                .as_deref()
                .is_some_and(|test_url| self.headless_verify(test_url))
            {
                finding_severity = SeverityLevel::High;
                evidence.strength = EvidenceStrength::Conclusive;
                evidence.summary =
                    "Headless verification confirmed JavaScript execution for reflected payload"
                        .to_string();
            }

            let candidate = CandidateFinding::new(
                scan_id,
                ActiveVulnType::ReflectedXss,
                payload_response.request_url.clone(),
                parameter,
                payload_response.http_method.clone(),
                payload_def.payload.clone(),
                payload_response.request_raw,
                body,
                payload_response.response_time_ms,
                payload_response.status_code,
                finding_severity,
                ExploitabilityLevel::Actionable,
                vec![evidence],
            );

            if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                let is_certain = finding.confidence == ConfidenceLevel::Certain;
                findings.push(finding);

                if is_certain {
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
