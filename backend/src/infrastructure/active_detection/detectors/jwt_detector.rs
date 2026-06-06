use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct JwtDetector {
    client: Client,
}

impl JwtDetector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VulnDetector for JwtDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![
            ActiveVulnType::JwtNoneAlgorithm,
            ActiveVulnType::JwtWeakSecret,
        ]
    }

    async fn detect(
        &self,
        target_url: &str,
        _parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>,
        _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
        _endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        _insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();

        // Phase 1: None algorithm bypass
        let none_payloads = payload_selector.select(ActiveVulnType::JwtNoneAlgorithm);
        for payload_def in &none_payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }
            let start = std::time::Instant::now();
            let mut req = self
                .client
                .get(target_url)
                .header("Authorization", format!("Bearer {}", payload_def.payload));
            if payload_selector.is_waf_bypass_enabled() {
                req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(
                    req,
                );
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            let status = resp.status().as_u16();

            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            // If we get 200 OK with the none-alg token, the endpoint accepts it
            if status == 200 && !body.contains("invalid") && !body.contains("unauthorized") {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::JwtNoneAlgorithm,
                    target_url: target_url.to_string(),
                    affected_parameter: "Authorization header".to_string(),
                    http_method: "GET".to_string(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!(
                            "GET {} HTTP/1.1\nAuthorization: Bearer {}",
                            target_url, payload_def.payload
                        ),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("JWT none-alg accepted (HTTP {})", status),
                        additional_notes: vec![
                            payload_def.description.clone(),
                            "Server accepted JWT with 'none' algorithm — critical auth bypass"
                                .to_string(),
                        ],
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

