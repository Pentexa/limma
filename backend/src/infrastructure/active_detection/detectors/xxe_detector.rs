use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{SeverityLevel, ConfidenceLevel};
use crate::infrastructure::active_detection::payloads::PayloadDatabase;

pub struct XxeDetector {
    client: Client,
    #[allow(dead_code)]
    payload_db: Arc<PayloadDatabase>,
}

impl XxeDetector {
    pub fn new(client: Client, payload_db: Arc<PayloadDatabase>) -> Self {
        Self { client, payload_db }
    }
}

#[async_trait]
impl VulnDetector for XxeDetector {
    fn supported_types(&self) -> Vec<ActiveVulnType> {
        vec![ActiveVulnType::XmlExternalEntity]
    }

    async fn detect(&self, target_url: &str, _parameter: &str, scan_id: Uuid, payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector, rate_limit_ms: u64, waf_monitor: std::sync::Arc<crate::infrastructure::safety::waf_monitor::WafMonitor>, _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::XmlExternalEntity);

        for payload_def in &payloads {
            if rate_limit_ms > 0 { tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await; }
            let start = std::time::Instant::now();
            let mut req = self.client.post(target_url).header("Content-Type", "application/xml").body(payload_def.payload.clone());
            if payload_selector.is_waf_bypass_enabled() { req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(req); }
            let resp = req.send().await.map_err(|e| e.to_string())?;

            let status = resp.status().as_u16();
            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            let file_indicators = ["root:x:0:0:", "root:$", "[fonts]", "ami-id", "PD9waH"];
            let detected = file_indicators.iter().any(|ind| body.contains(ind));

            if detected {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(), scan_id, timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::XmlExternalEntity,
                    target_url: target_url.to_string(), affected_parameter: "XML body".to_string(),
                    http_method: "POST".to_string(), payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("POST {} HTTP/1.1\nContent-Type: application/xml\n\n{}", target_url, payload_def.payload),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("XXE file content detected: {}", payload_def.id),
                        additional_notes: vec![payload_def.description.clone()],
                    },
                    severity: SeverityLevel::Critical, confidence: ConfidenceLevel::Certain,
                    exploitability: ExploitabilityLevel::Actionable,
                    poc_generated: false, poc_id: None, verified: true, false_positive: false,
                });
                break;
            }
        }

        Ok(findings)
    }
}
