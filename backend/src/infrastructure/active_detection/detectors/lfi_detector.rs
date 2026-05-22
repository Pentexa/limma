use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

pub struct LfiDetector {
    client: Client,
    
    
}

impl LfiDetector {
    pub fn new(client: Client, ) -> Self {
        Self { client,  }
    }

    fn check_file_content(body: &str) -> Option<&'static str> {
        let indicators: Vec<(&str, &str)> = vec![
            ("root:x:0:0:", "/etc/passwd"),
            ("root:$", "/etc/shadow"),
            ("[fonts]", "win.ini"),
            ("[extensions]", "win.ini"),
            ("PD9waH", "PHP source (base64)"),
        ];
        for (pattern, file) in indicators {
            if body.contains(pattern) {
                return Some(file);
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
        _baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();
        let payloads = payload_selector.select(ActiveVulnType::LocalFileInclusion);

        for payload_def in &payloads {
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
            let resp = req.send().await.map_err(|e| e.to_string())?;

            let status = resp.status().as_u16();
            waf_monitor.register_response(target_url, status);
            if waf_monitor.is_waf_detected(target_url) {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
            }

            let elapsed = start.elapsed().as_millis() as u64;
            let body = resp.text().await.map_err(|e| e.to_string())?;

            if let Some(file) = Self::check_file_content(&body) {
                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::LocalFileInclusion,
                    target_url: target_url.to_string(),
                    affected_parameter: parameter.to_string(),
                    http_method: "GET".to_string(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("GET {} HTTP/1.1", test_url),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("File content detected: {}", file),
                        additional_notes: vec![
                            payload_def.description.clone(),
                            format!("Sensitive file read: {}", file),
                        ],
                    },
                    severity: SeverityLevel::High,
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

        Ok(findings)
    }
}
