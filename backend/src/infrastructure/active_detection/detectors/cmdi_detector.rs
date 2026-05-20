use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{SeverityLevel, ConfidenceLevel};
use crate::infrastructure::active_detection::payloads::PayloadDatabase;

pub struct CmdiDetector {
    client: Client,
    #[allow(dead_code)]
    payload_db: Arc<PayloadDatabase>,
}

impl CmdiDetector {
    pub fn new(client: Client, payload_db: Arc<PayloadDatabase>) -> Self {
        Self { client, payload_db }
    }

    fn check_cmd_output(body: &str) -> bool {
        let indicators = ["uid=", "root:", "www-data", "[fonts]", "Windows", "SYSTEM"];
        indicators.iter().any(|ind| body.contains(ind))
    }
}

#[async_trait]
impl VulnDetector for CmdiDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::CommandInjection, ActiveVulnType::CommandInjectionBlind]
    }

    async fn detect(&self, target_url: &str, parameter: &str, scan_id: Uuid, payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector, rate_limit_ms: u64, waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>, _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();

        // Direct output detection
        let payloads = payload_selector.select(ActiveVulnType::CommandInjection);
        for payload_def in &payloads {
            if rate_limit_ms > 0 { tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await; }
            let test_url = format!("{}?{}={}", target_url, parameter, urlencoding::encode(&payload_def.payload));
            let start = std::time::Instant::now();
            let mut req = self.client.get(&test_url);
            if payload_selector.is_waf_bypass_enabled() { req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(req); }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            
            let status = resp.status().as_u16();
            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            if Self::check_cmd_output(&body) {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(), scan_id, timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::CommandInjection,
                    target_url: target_url.to_string(), affected_parameter: parameter.to_string(),
                    http_method: "GET".to_string(), payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("GET {} HTTP/1.1", test_url),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("Command output detected: {}", payload_def.id),
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: SeverityLevel::Critical, confidence: ConfidenceLevel::Certain,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false, poc_id: None, verified: true, false_positive: false,
                });
                break;
            }
        }

        // Blind (time-based) detection
        if findings.is_empty() {
            let blind_payloads = payload_selector.select(ActiveVulnType::CommandInjectionBlind);
            for payload_def in &blind_payloads {
                if rate_limit_ms > 0 { tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await; }
                let test_url = format!("{}?{}={}", target_url, parameter, urlencoding::encode(&payload_def.payload));
                let start = std::time::Instant::now();
                let mut req = self.client.get(&test_url);
            if payload_selector.is_waf_bypass_enabled() { req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(req); }
            let resp = req.send().await.map_err(|e| e.to_string())?;
                
                let status = resp.status().as_u16();
                waf_monitor.register_response(target_url, status);
                if waf_monitor.is_waf_detected(target_url) {
                    tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
                }

                let elapsed = start.elapsed().as_millis() as u64;

                if elapsed >= 4500 {
                    findings.push(ActiveVulnFinding {
                        id: Uuid::new_v4(), scan_id, timestamp: Utc::now(),
                        vuln_type: ActiveVulnType::CommandInjectionBlind,
                        target_url: target_url.to_string(), affected_parameter: parameter.to_string(),
                        http_method: "GET".to_string(), payload_used: payload_def.payload.clone(),
                        evidence: ActiveVulnEvidence {
                            request_raw: format!("GET {} HTTP/1.1", test_url),
                            response_raw: format!("Response delayed: {}ms", elapsed),
                            response_time_ms: elapsed,
                            matched_indicator: format!("Blind CMDi time delay: {}ms", elapsed),
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
