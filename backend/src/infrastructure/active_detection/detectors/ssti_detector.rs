#![allow(clippy::collapsible_match)]
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::ConfidenceLevel;

pub struct SstiDetector {
    client: Client,
}

impl SstiDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
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
        _endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        _insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::ServerSideTemplateInjection);

        for payload_def in payloads {
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

            let resp = match req.send().await {
                Ok(r) => r,
                Err(_) => continue,
            };

            let elapsed = start.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();

            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let body = resp.text().await.unwrap_or_default();

            let mut is_vuln = false;
            let mut matched_ind = String::new();

            match &payload_def.expected_indicator {
                ExpectedIndicator::ReflectedContent(content) => {
                    if body.contains(content) {
                        is_vuln = true;
                        matched_ind = format!("Template evaluated and reflected: {}", content);
                    }
                }
                ExpectedIndicator::ErrorPattern(pattern) => {
                    if body.contains(pattern) {
                        is_vuln = true;
                        matched_ind = format!("SSTI Error pattern matched: {}", pattern);
                    }
                }
                _ => {}
            }

            if is_vuln {
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
                    vuln_type: ActiveVulnType::ServerSideTemplateInjection,
                    target_url: target_url.to_string(),
                    affected_parameter: parameter.to_string(),
                    http_method: "GET".to_string(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("GET {} HTTP/1.1", test_url),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: matched_ind,
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: payload_def.severity,
                    confidence: ConfidenceLevel::Certain,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: true,
                    false_positive: false,
                });
            }
        }

        Ok(findings)
    }
}

