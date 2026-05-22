use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct RedirectDetector {
    _client: Client,
}

impl RedirectDetector {
    pub fn new(client: Client) -> Self {
        Self { _client: client }
    }
}

#[async_trait]
impl VulnDetector for RedirectDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::OpenRedirect]
    }

    async fn detect(
        &self,
        target_url: &str,
        parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>,
        _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::OpenRedirect);

        // Use a non-following client for redirect detection
        let no_redirect_client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        for payload_def in &payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }
            let test_url = format!(
                "{}?{}={}",
                target_url,
                parameter,
                urlencoding::encode(&payload_def.payload)
            );
            let start = std::time::Instant::now();
            let mut req = no_redirect_client.get(&test_url);
            if payload_selector.is_waf_bypass_enabled() {
                req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(
                    req,
                );
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;

            let status = resp.status().as_u16();
            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();

            if (300..400).contains(&status) {
                if let Some(location) = resp.headers().get("location") {
                    let loc_str = location.to_str().unwrap_or("");
                    if loc_str.contains("evil.com")
                        || loc_str.starts_with("javascript:")
                        || loc_str.starts_with("//")
                    {
                        findings.push(ActiveVulnFinding {
                            id: Uuid::new_v4(),
                            scan_id,
                            timestamp: Utc::now(),
                            vuln_type: ActiveVulnType::OpenRedirect,
                            target_url: target_url.to_string(),
                            affected_parameter: parameter.to_string(),
                            http_method: "GET".to_string(),
                            payload_used: payload_def.payload.clone(),
                            evidence: ActiveVulnEvidence {
                                request_raw: format!("GET {} HTTP/1.1", test_url),
                                response_raw: format!("HTTP/1.1 {}\nLocation: {}", status, loc_str),
                                response_time_ms: elapsed,
                                matched_indicator: format!("Redirect to external: {}", loc_str),
                                additional_notes: vec![payload_def.description.clone()],
                            },
                            severity: SeverityLevel::Medium,
                            confidence: ConfidenceLevel::Certain,
                            exploitability: ExploitabilityLevel::Actionable,
                            poc_generated: false,
                            poc_id: None,
                            verified: true,
                            false_positive: false,
                        });
                        break;
                    }
                }
            }
        }

        Ok(findings)
    }
}
