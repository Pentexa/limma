use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct SsrfDetector {
    client: Client,
}

impl SsrfDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn check_cloud_metadata(body: &str) -> Option<&'static str> {
        let indicators = [
            ("ami-id", "AWS EC2 Metadata"),
            ("instance-id", "AWS EC2 Metadata"),
            ("computeMetadata", "GCP Metadata"),
            ("\"compute\"", "Azure IMDS"),
        ];
        for (pattern, cloud) in indicators {
            if body.contains(pattern) {
                return Some(cloud);
            }
        }
        None
    }
}

#[async_trait]
impl VulnDetector for SsrfDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::ServerSideRequestForgery]
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
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::ServerSideRequestForgery);

        // First: get baseline response
        let baseline_resp = super::send_payload_request(
            &self.client,
            target_url,
            parameter,
            "https://example.com",
            endpoint_ctx,
            insertion_point,
            false,
        )
        .await?;
        let baseline_len = baseline_resp.response_body.len();

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

            let status = payload_response.status_code;
            waf_monitor.register_response(&payload_response.request_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = payload_response.response_time_ms;
            let body = payload_response.response_body.clone();

            let mut detected = false;
            let mut indicator_msg = String::new();

            // Check for cloud metadata
            if let Some(cloud) = Self::check_cloud_metadata(&body) {
                detected = true;
                indicator_msg = format!("Cloud metadata exposed: {}", cloud);
            }

            // Check if response differs significantly from baseline (internal page loaded)
            if !detected && status == 200 && body.len() > baseline_len * 2 {
                detected = true;
                indicator_msg = format!(
                    "Response size anomaly: {} vs baseline {}",
                    body.len(),
                    baseline_len
                );
            }

            if detected {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::ServerSideRequestForgery,
                    target_url: payload_response.request_url.clone(),
                    affected_parameter: parameter.to_string(),
                    http_method: payload_response.http_method.clone(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: payload_response.request_raw,
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: indicator_msg,
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: SeverityLevel::Critical,
                    confidence: ConfidenceLevel::Firm,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: false,
                    false_positive: false,
                });
                break;
            }
        }

        Ok(findings)
    }
}
