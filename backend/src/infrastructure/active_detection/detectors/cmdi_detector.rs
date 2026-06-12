use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct CmdiDetector {
    client: Client,
}

impl CmdiDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn check_cmd_output(body: &str) -> bool {
        let indicators = ["uid=", "root:", "www-data", "[fonts]", "Windows", "SYSTEM"];
        indicators.iter().any(|ind| body.contains(ind))
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
        _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
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

            if Self::check_cmd_output(&body) {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::CommandInjection,
                    target_url: payload_response.request_url.clone(),
                    affected_parameter: parameter.to_string(),
                    http_method: payload_response.http_method.clone(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: payload_response.request_raw,
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: payload_response.response_time_ms,
                        matched_indicator: format!("Command output detected: {}", payload_def.id),
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: SeverityLevel::Critical,
                    confidence: ConfidenceLevel::Certain,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false,
                    poc_id: None,
                    verified: true,
                    false_positive: false,
                });
                break;
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

                if payload_response.response_time_ms >= 4500 {
                    findings.push(ActiveVulnFinding {
                        id: Uuid::new_v4(), scan_id, timestamp: Utc::now(),
                        vuln_type: ActiveVulnType::CommandInjectionBlind,
                        target_url: payload_response.request_url.clone(), affected_parameter: parameter.to_string(),
                        http_method: payload_response.http_method.clone(), payload_used: payload_def.payload.clone(),
                        evidence: ActiveVulnEvidence {
                            request_raw: payload_response.request_raw,
                            response_raw: format!("Response delayed: {}ms", payload_response.response_time_ms),
                            response_time_ms: payload_response.response_time_ms,
                            matched_indicator: format!("Blind CMDi time delay: {}ms", payload_response.response_time_ms),
                            additional_notes: vec!["Time-based blind command injection".to_string()],
                        },
                        severity: SeverityLevel::Critical, confidence: ConfidenceLevel::Firm,
                        exploitability: ExploitabilityLevel::Conditional,
                        poc_generated: false, poc_id: None, verified: false, false_positive: false,
                    });
                    break;
                }
            }
        }

        Ok(findings)
    }
}
