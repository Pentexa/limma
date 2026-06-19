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

pub struct XxeDetector {
    client: Client,
}

impl XxeDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VulnDetector for XxeDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::XmlExternalEntity]
    }

    async fn detect(
        &self,
        target_url: &str,
        _parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>,
        baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        _insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::XmlExternalEntity);

        for payload_def in &payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }
            let request_url = endpoint_ctx
                .map(|ctx| ctx.url.clone())
                .unwrap_or_else(|| target_url.to_string());
            let method = endpoint_ctx
                .map(|ctx| ctx.method.to_uppercase())
                .filter(|method| matches!(method.as_str(), "POST" | "PUT" | "PATCH"))
                .unwrap_or_else(|| "POST".to_string());

            let start = std::time::Instant::now();
            let mut req = match method.as_str() {
                "PUT" => self.client.put(&request_url),
                "PATCH" => self.client.patch(&request_url),
                _ => self.client.post(&request_url),
            };
            if let Some(ctx) = endpoint_ctx {
                for (k, v) in &ctx.headers {
                    req = req.header(k, v);
                }
            }
            req = req
                .header("Content-Type", "application/xml")
                .body(payload_def.payload.clone());
            if payload_selector.is_waf_bypass_enabled() {
                req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(
                    req,
                );
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;

            let status = resp.status().as_u16();
            waf_monitor.register_response(&request_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            let evidence = match &payload_def.expected_indicator {
                ExpectedIndicator::FileContent(indicator)
                | ExpectedIndicator::ReflectedContent(indicator) => {
                    TokenMatcher::find_expected(&body, indicator).map(|mut evidence| {
                        evidence.summary =
                            format!("XXE response exposed expected content: {}", indicator);
                        evidence
                    })
                }
                ExpectedIndicator::ErrorPattern(pattern) => {
                    if body.contains(pattern) {
                        Some(EvidenceItem::error_pattern(
                            pattern.clone(),
                            format!("XXE parser error pattern matched: {}", pattern),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            }
            .or_else(|| {
                TokenMatcher::find_any(
                    &body,
                    &["root:x:0:0:", "root:$", "[fonts]", "ami-id", "PD9waH"],
                )
            });

            if let Some(evidence) = evidence {
                let request_raw = format!(
                    "{} {} HTTP/1.1\nContent-Type: application/xml\n\n{}",
                    method, request_url, payload_def.payload
                );
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::XmlExternalEntity,
                    request_url.clone(),
                    "XML body",
                    method.clone(),
                    payload_def.payload.clone(),
                    request_raw,
                    body,
                    elapsed,
                    status,
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
