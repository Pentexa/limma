use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::infrastructure::active_detection::evidence::token_matcher::TokenMatcher;
use crate::infrastructure::active_detection::evidence::EvidenceItem;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

pub struct LfiDetector {
    client: Client,
}

impl LfiDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn check_file_content(body: &str) -> Option<(&'static str, &'static str)> {
        let indicators: Vec<(&str, &str)> = vec![
            ("root:x:0:0:", "/etc/passwd"),
            ("root:$", "/etc/shadow"),
            ("[fonts]", "win.ini"),
            ("[extensions]", "win.ini"),
            ("PD9waH", "PHP source (base64)"),
        ];
        for (pattern, file) in indicators {
            if body.contains(pattern) {
                return Some((pattern, file));
            }
        }
        None
    }
}

#[async_trait]
impl VulnDetector for LfiDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![
            ActiveVulnType::LocalFileInclusion,
            ActiveVulnType::RemoteFileInclusion,
            ActiveVulnType::PathTraversal,
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
        let payloads = payload_selector.select(ActiveVulnType::LocalFileInclusion);

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

            let evidence = match &payload_def.expected_indicator {
                ExpectedIndicator::FileContent(indicator)
                | ExpectedIndicator::ReflectedContent(indicator) => {
                    TokenMatcher::find_expected(&body, indicator).map(|mut evidence| {
                        evidence.summary =
                            format!("Sensitive file content detected: {}", indicator);
                        evidence
                    })
                }
                _ => None,
            }
            .or_else(|| {
                Self::check_file_content(&body).map(|(indicator, file)| {
                    EvidenceItem::file_content(
                        indicator,
                        format!("Sensitive file read confirmed: {}", file),
                    )
                })
            });

            if let Some(evidence) = evidence {
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::LocalFileInclusion,
                    payload_response.request_url.clone(),
                    parameter,
                    payload_response.http_method.clone(),
                    payload_def.payload.clone(),
                    payload_response.request_raw,
                    body,
                    payload_response.response_time_ms,
                    payload_response.status_code,
                    payload_def.severity.clone(),
                    ExploitabilityLevel::Actionable,
                    vec![evidence],
                );

                if let Some(finding) = VerificationPipeline::verify(candidate, baseline) {
                    findings.push(finding);
                    break;
                }
            }
        }

        Ok(findings)
    }
}
