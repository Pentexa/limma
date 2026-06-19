use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::SeverityLevel;
use crate::infrastructure::active_detection::evidence::token_matcher::TokenMatcher;
use crate::infrastructure::active_detection::evidence::EvidenceItem;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

pub struct CmdiDetector {
    client: Client,
}

impl CmdiDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VulnDetector for CmdiDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![
            ActiveVulnType::CommandInjection,
            ActiveVulnType::CommandInjectionBlind,
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

        // Direct output detection
        let payloads = payload_selector.select(ActiveVulnType::CommandInjection);
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

            if let Some(evidence) = TokenMatcher::find_any(
                &body,
                &["uid=", "root:", "www-data", "[fonts]", "Windows", "SYSTEM"],
            ) {
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::CommandInjection,
                    payload_response.request_url.clone(),
                    parameter,
                    payload_response.http_method.clone(),
                    payload_def.payload.clone(),
                    payload_response.request_raw,
                    body,
                    payload_response.response_time_ms,
                    payload_response.status_code,
                    SeverityLevel::Critical,
                    ExploitabilityLevel::Actionable,
                    vec![evidence],
                );

                if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                    findings.push(finding);
                    break;
                }
            }
        }

        // Blind (time-based) detection
        if findings.is_empty() {
            let blind_payloads = payload_selector.select(ActiveVulnType::CommandInjectionBlind);
            for payload_def in &blind_payloads {
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

                let expected_delay_ms = match &payload_def.expected_indicator {
                    ExpectedIndicator::TimeDelay(delay) => *delay,
                    _ => 5_000,
                };
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::CommandInjectionBlind,
                    payload_response.request_url.clone(),
                    parameter,
                    payload_response.http_method.clone(),
                    payload_def.payload.clone(),
                    payload_response.request_raw,
                    format!("Response delayed: {}ms", payload_response.response_time_ms),
                    payload_response.response_time_ms,
                    payload_response.status_code,
                    SeverityLevel::Critical,
                    ExploitabilityLevel::Conditional,
                    vec![EvidenceItem::time_delay(
                        payload_response.response_time_ms,
                        expected_delay_ms,
                    )],
                )
                .with_expected_delay(expected_delay_ms);

                if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                    findings.push(finding);
                    break;
                }
            }
        }

        Ok(findings)
    }
}
