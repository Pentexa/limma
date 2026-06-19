use async_trait::async_trait;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::infrastructure::active_detection::evidence::EvidenceItem;
use crate::infrastructure::active_detection::verification::{
    CandidateFinding, VerificationPipeline,
};

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
        baseline: Option<&crate::infrastructure::active_detection::differential::BaselineProfile>,
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        _insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();

        let none_payloads = payload_selector.select(ActiveVulnType::JwtNoneAlgorithm);
        for payload_def in &none_payloads {
            if rate_limit_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms)).await;
            }
            let request_url = endpoint_ctx
                .map(|ctx| ctx.url.clone())
                .unwrap_or_else(|| target_url.to_string());
            let method = endpoint_ctx
                .map(|ctx| ctx.method.to_uppercase())
                .unwrap_or_else(|| "GET".to_string());

            let start = std::time::Instant::now();
            let mut req = match method.as_str() {
                "POST" => self.client.post(&request_url),
                "PUT" => self.client.put(&request_url),
                "PATCH" => self.client.patch(&request_url),
                "DELETE" => self.client.delete(&request_url),
                _ => self.client.get(&request_url),
            };
            if let Some(ctx) = endpoint_ctx {
                for (k, v) in &ctx.headers {
                    req = req.header(k, v);
                }
                if method != "GET" {
                    if let Some(body) = &ctx.body {
                        req = req.body(body.clone());
                    }
                }
            }
            req = req.header("Authorization", format!("Bearer {}", payload_def.payload));
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
            let body_lower = body.to_lowercase();

            if status == 200
                && !body_lower.contains("invalid")
                && !body_lower.contains("unauthorized")
            {
                let request_raw = format!(
                    "{} {} HTTP/1.1\nAuthorization: Bearer {}",
                    method, request_url, payload_def.payload
                );
                let candidate = CandidateFinding::new(
                    scan_id,
                    ActiveVulnType::JwtNoneAlgorithm,
                    request_url.clone(),
                    "Authorization header",
                    method.clone(),
                    payload_def.payload.clone(),
                    request_raw,
                    body,
                    elapsed,
                    status,
                    payload_def.severity.clone(),
                    ExploitabilityLevel::Actionable,
                    vec![EvidenceItem::jwt_accepted(
                        status,
                        format!("JWT none-alg accepted (HTTP {})", status),
                    )],
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
