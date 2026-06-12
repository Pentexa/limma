#![allow(clippy::collapsible_match)]
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::ConfidenceLevel;

pub struct IdorDetector {
    client: Client,
}

impl IdorDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VulnDetector for IdorDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::InsecureDirectObjectReference]
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
        let payloads = payload_selector.select(ActiveVulnType::InsecureDirectObjectReference);

        for payload_def in payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }

            let payload_response = match super::send_payload_request(
                &self.client,
                target_url,
                parameter,
                &payload_def.payload,
                endpoint_ctx,
                insertion_point,
                payload_selector.is_waf_bypass_enabled(),
            )
            .await
            {
                Ok(r) => r,
                Err(_) => continue,
            };

            let elapsed = payload_response.response_time_ms;
            let status = payload_response.status_code;

            waf_monitor.register_response(&payload_response.request_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let body = payload_response.response_body.clone();

            let mut is_vuln = false;
            let mut matched_ind = String::new();

            match &payload_def.expected_indicator {
                ExpectedIndicator::StatusCode(code) => {
                    if status == *code {
                        // Very naive IDOR check: if it returns the expected status code (e.g., 200 OK)
                        // This usually requires more context (e.g., checking if data actually belongs to another user),
                        // but this is the initial phase detection.
                        is_vuln = true;
                        matched_ind = format!("Status code matched: {}", code);
                    }
                }
                ExpectedIndicator::ErrorPattern(pattern) => {
                    if body.contains(pattern) {
                        is_vuln = true;
                        matched_ind = format!("Error pattern matched: {}", pattern);
                    }
                }
                _ => {}
            }

            if is_vuln {
                // If it's just a 200 OK, it could be a false positive (e.g., the page always returns 200).
                let is_false_positive = baseline
                    .map(|b| b.contains_indicator(&payload_def.payload))
                    .unwrap_or(false);
                if is_false_positive {
                    continue;
                }

                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::InsecureDirectObjectReference,
                    target_url: payload_response.request_url.clone(),
                    affected_parameter: parameter.to_string(),
                    http_method: payload_response.http_method.clone(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: payload_response.request_raw,
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: matched_ind,
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: payload_def.severity,
                    confidence: ConfidenceLevel::Tentative, // IDOR detections usually have lower confidence without auth context
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: false,
                    false_positive: false,
                });
            }
        }

        Ok(findings)
    }
}
