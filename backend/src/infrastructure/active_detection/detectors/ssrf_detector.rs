use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::infrastructure::active_detection::evidence::response_diff::ResponseDiffAnalyzer;
use crate::infrastructure::active_detection::evidence::token_matcher::TokenMatcher;
use crate::infrastructure::active_detection::evidence::{EvidenceItem, EvidenceStrength};
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

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
        baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::ServerSideRequestForgery);

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
        let baseline_snapshot =
            ResponseDiffAnalyzer::snapshot(baseline_resp.status_code, baseline_resp.response_body);

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
            let mut evidences = Vec::new();

            if let Some(cloud) = Self::check_cloud_metadata(&body) {
                if let Some(mut evidence) = TokenMatcher::find_any(
                    &body,
                    &["ami-id", "instance-id", "computeMetadata", "\"compute\""],
                ) {
                    evidence.summary = format!("Cloud metadata exposed: {}", cloud);
                    evidences.push(evidence);
                }
            }

            if evidences.is_empty() && status == 200 {
                let observed_snapshot = ResponseDiffAnalyzer::snapshot(status, body.clone());
                let diff =
                    ResponseDiffAnalyzer::compare_snapshots(&baseline_snapshot, &observed_snapshot);

                if diff.is_significant()
                    && observed_snapshot.content_length > baseline_snapshot.content_length * 2
                {
                    let mut evidence = EvidenceItem::response_diff(
                        "ssrf_internal_response_diff",
                        format!(
                            "SSRF response differed from safe URL baseline: {}",
                            diff.summary()
                        ),
                    );
                    evidence.strength = EvidenceStrength::Conclusive;
                    evidences.push(evidence);
                }
            }

            if evidences.is_empty() {
                continue;
            }

            let candidate = CandidateFinding::new(
                scan_id,
                ActiveVulnType::ServerSideRequestForgery,
                payload_response.request_url.clone(),
                parameter,
                payload_response.http_method.clone(),
                payload_def.payload.clone(),
                payload_response.request_raw,
                body,
                elapsed,
                status,
                payload_def.severity.clone(),
                ExploitabilityLevel::Actionable,
                evidences,
            );

            if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                findings.push(finding);
                break;
            }
        }

        Ok(findings)
    }
}
