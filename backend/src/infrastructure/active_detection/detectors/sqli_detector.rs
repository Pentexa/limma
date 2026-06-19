use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::SeverityLevel;
use crate::infrastructure::active_detection::evidence::error_pattern_matcher::ErrorPatternMatcher;
use crate::infrastructure::active_detection::evidence::response_diff::ResponseDiffAnalyzer;
use crate::infrastructure::active_detection::evidence::EvidenceItem;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

pub struct SqliDetector {
    client: Client,
    error_matcher: ErrorPatternMatcher,
}

impl SqliDetector {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            error_matcher: ErrorPatternMatcher::new(),
        }
    }
}

#[async_trait]
impl VulnDetector for SqliDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![
            ActiveVulnType::SqlInjectionError,
            ActiveVulnType::SqlInjectionUnion,
            ActiveVulnType::SqlInjectionBlindTime,
            ActiveVulnType::SqlInjectionBlindBoolean,
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

        // Phase 1: Error-based detection. The detector emits a candidate; the
        // pipeline rejects it if the same error already exists in baseline.
        let error_payloads = payload_selector.select(ActiveVulnType::SqlInjectionError);
        for payload_def in &error_payloads {
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

            if let Some(matched) = self.error_matcher.match_sql(&body) {
                let evidence = EvidenceItem::error_pattern(
                    matched.matched_text.clone(),
                    format!(
                        "SQL error from {} DB: {}",
                        matched.family, matched.matched_text
                    ),
                );
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::SqlInjectionError,
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

        // Phase 1.5: Boolean blind SQLi via baseline/true/false differential.
        if findings.is_empty() {
            if let Some(base) = baseline {
                let true_payload = "1' AND 1=1-- -";
                let false_payload = "1' AND 1=2-- -";

                let true_response = super::send_payload_request(
                    &self.client,
                    target_url,
                    parameter,
                    true_payload,
                    endpoint_ctx,
                    insertion_point,
                    payload_selector.is_waf_bypass_enabled(),
                )
                .await;
                let false_response = super::send_payload_request(
                    &self.client,
                    target_url,
                    parameter,
                    false_payload,
                    endpoint_ctx,
                    insertion_point,
                    payload_selector.is_waf_bypass_enabled(),
                )
                .await;

                if let (Ok(true_resp), Ok(false_resp)) = (true_response, false_response) {
                    waf_monitor.register_response(&true_resp.request_url, true_resp.status_code);
                    waf_monitor.register_response(&false_resp.request_url, false_resp.status_code);

                    let true_diff = ResponseDiffAnalyzer::compare_to_baseline(
                        base,
                        true_resp.status_code,
                        &true_resp.response_body,
                    );
                    let false_diff = ResponseDiffAnalyzer::compare_to_baseline(
                        base,
                        false_resp.status_code,
                        &false_resp.response_body,
                    );

                    if !true_diff.is_significant() && false_diff.is_significant() {
                        let response_summary = format!(
                            "True diff: {}; False diff: {}; true_length={}, false_length={}, baseline_length={}",
                            true_diff.summary(),
                            false_diff.summary(),
                            true_resp.response_body.len(),
                            false_resp.response_body.len(),
                            base.content_length
                        );
                        let candidate = CandidateFinding::new(
                            scan_id,
                            ActiveVulnType::SqlInjectionBlindBoolean,
                            true_resp.request_url.clone(),
                            parameter,
                            true_resp.http_method.clone(),
                            true_payload,
                            format!("{}\n\n{}", true_resp.request_raw, false_resp.request_raw),
                            response_summary,
                            true_resp.response_time_ms + false_resp.response_time_ms,
                            true_resp.status_code,
                            SeverityLevel::Critical,
                            ExploitabilityLevel::Actionable,
                            vec![EvidenceItem::response_diff(
                                "boolean_true_false_response_diff",
                                "Boolean blind SQL injection verified via baseline/true/false response differential",
                            )],
                        );

                        if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                            findings.push(finding);
                        }
                    }
                }
            }
        }

        // Phase 2: Time-based blind SQLi with baseline-aware delay analysis.
        if findings.is_empty() {
            let time_payloads = payload_selector.select(ActiveVulnType::SqlInjectionBlindTime);
            for payload_def in &time_payloads {
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
                    ActiveVulnType::SqlInjectionBlindTime,
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
