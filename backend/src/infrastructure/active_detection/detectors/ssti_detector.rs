#![allow(clippy::collapsible_match)]
use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::infrastructure::active_detection::evidence::error_pattern_matcher::ErrorPatternMatcher;
use crate::infrastructure::active_detection::evidence::token_matcher::TokenMatcher;
use crate::infrastructure::active_detection::evidence::EvidenceItem;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

pub struct SstiDetector {
    client: Client,
    error_matcher: ErrorPatternMatcher,
}

impl SstiDetector {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            error_matcher: ErrorPatternMatcher::new(),
        }
    }
}

#[async_trait]
impl VulnDetector for SstiDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::ServerSideTemplateInjection]
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
        let payloads = payload_selector.select(ActiveVulnType::ServerSideTemplateInjection);

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

            let status = payload_response.status_code;
            let elapsed = payload_response.response_time_ms;
            waf_monitor.register_response(&payload_response.request_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let body = payload_response.response_body.clone();

            let mut evidences = Vec::new();

            match &payload_def.expected_indicator {
                ExpectedIndicator::ReflectedContent(content) => {
                    if let Some(mut evidence) = TokenMatcher::find_expected(&body, content) {
                        evidence.summary =
                            format!("Template expression evaluated and reflected: {}", content);
                        evidences.push(evidence);
                    }
                }
                ExpectedIndicator::ErrorPattern(pattern) => {
                    if body.contains(pattern) {
                        evidences.push(EvidenceItem::error_pattern(
                            pattern.clone(),
                            format!("SSTI error/leak pattern matched: {}", pattern),
                        ));
                    } else if let Some(matched) = self.error_matcher.match_framework(&body) {
                        evidences.push(matched.to_evidence());
                    }
                }
                _ => {}
            }

            if evidences.is_empty() {
                continue;
            }

            let candidate = CandidateFinding::new(
                scan_id,
                ActiveVulnType::ServerSideTemplateInjection,
                payload_response.request_url.clone(),
                parameter,
                payload_response.http_method.clone(),
                payload_def.payload.clone(),
                payload_response.request_raw,
                body,
                elapsed,
                status,
                payload_def.severity,
                ExploitabilityLevel::Actionable,
                evidences,
            );

            if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                findings.push(finding);
            }
        }

        Ok(findings)
    }
}
