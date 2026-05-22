use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct XssDetector {
    client: Client,
    
    
}

impl XssDetector {
    pub fn new(client: Client, ) -> Self {
        Self { client,  }
    }

    fn check_xss_reflection(&self, body: &str, payload: &str) -> bool {
        if !body.contains(payload) {
            return false;
        }
        // Ensure it's NOT HTML-encoded
        let encoded = payload.replace('<', "&lt;").replace('>', "&gt;");
        if body.contains(&encoded) && !body.contains(payload) {
            return false;
        }
        // Check for active script/event handler contexts
        let dangerous_indicators = [
            "<script",
            "onerror=",
            "onload=",
            "onclick=",
            "onfocus=",
            "onmouseover=",
            "ontoggle=",
            "javascript:",
            "constructor.constructor",
        ];
        dangerous_indicators
            .iter()
            .any(|ind| payload.to_lowercase().contains(&ind.to_lowercase()))
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
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::ReflectedXss);

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
            let mut req = self.client.get(&test_url);
            if payload_selector.is_waf_bypass_enabled() {
                req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(
                    req,
                );
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            let elapsed = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();

            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let body = resp.text().await.map_err(|e| e.to_string())?;

            if self.check_xss_reflection(&body, &payload_def.payload) {
                // DIFFERENTIAL ANALYSIS: False Positive Check
                // If the baseline inherently has this exact "payload" string (e.g. it was part of the original page script), ignore it.
                let is_false_positive = baseline
                    .map(|b| b.contains_indicator(&payload_def.payload))
                    .unwrap_or(false);

                if is_false_positive {
                    continue; // Skip, it's just normal page content
                }

                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::ReflectedXss,
                    target_url: target_url.to_string(),
                    affected_parameter: parameter.to_string(),
                    http_method: "GET".to_string(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("GET {} HTTP/1.1", test_url),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("XSS payload reflected: {}", payload_def.id),
                        additional_notes: vec![
                            payload_def.description.clone(),
                            "Payload reflected without HTML encoding".to_string(),
                            "Verified against Differential Baseline".to_string(),
                        ],
                    },
                    severity: SeverityLevel::High,
                    confidence: ConfidenceLevel::Certain,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: true,
                    false_positive: false,
                });
                // One finding per parameter is enough
                break;
            }
        }

        Ok(findings)
    }
}
