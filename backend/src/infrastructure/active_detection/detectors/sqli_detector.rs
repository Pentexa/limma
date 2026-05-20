use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};
use crate::infrastructure::active_detection::payloads::PayloadDatabase;

struct SqlErrorPattern {
    db_type: &'static str,
    patterns: Vec<Regex>,
}

pub struct SqliDetector {
    client: Client,
    #[allow(dead_code)]
    payload_db: Arc<PayloadDatabase>,
    error_patterns: Vec<SqlErrorPattern>,
}

impl SqliDetector {
    pub fn new(client: Client, payload_db: Arc<PayloadDatabase>) -> Self {
        let error_patterns = vec![
            SqlErrorPattern {
                db_type: "MySQL",
                patterns: vec![
                    Regex::new(r"(?i)SQL syntax.*MySQL").unwrap(),
                    Regex::new(r"(?i)Warning.*mysql_").unwrap(),
                    Regex::new(r"(?i)MySqlException").unwrap(),
                ],
            },
            SqlErrorPattern {
                db_type: "PostgreSQL",
                patterns: vec![
                    Regex::new(r"(?i)PostgreSQL.*ERROR").unwrap(),
                    Regex::new(r"(?i)Warning.*pg_").unwrap(),
                    Regex::new(r"(?i)Npgsql").unwrap(),
                ],
            },
            SqlErrorPattern {
                db_type: "MSSQL",
                patterns: vec![
                    Regex::new(r"(?i)Driver.*SQL[\-_ ]*Server").unwrap(),
                    Regex::new(r"(?i)OLE DB.*SQL Server").unwrap(),
                    Regex::new(r"(?i)SqlException").unwrap(),
                    Regex::new(r"(?i)Unclosed quotation mark").unwrap(),
                ],
            },
            SqlErrorPattern {
                db_type: "Oracle",
                patterns: vec![
                    Regex::new(r"(?i)ORA-[0-9]{5}").unwrap(),
                    Regex::new(r"(?i)Oracle error").unwrap(),
                    Regex::new(r"(?i)quoted string not properly terminated").unwrap(),
                ],
            },
            SqlErrorPattern {
                db_type: "SQLite",
                patterns: vec![
                    Regex::new(r"(?i)SQLite.*Exception").unwrap(),
                    Regex::new(r"(?i)Warning.*sqlite_").unwrap(),
                    Regex::new(r"(?i)SQLITE_ERROR").unwrap(),
                ],
            },
        ];
        Self {
            client,
            payload_db,
            error_patterns,
        }
    }

    fn identify_sql_error<'a>(&'a self, body: &str) -> Option<(&'a str, String)> {
        for pattern_set in &self.error_patterns {
            for regex in &pattern_set.patterns {
                if let Some(mat) = regex.find(body) {
                    return Some((pattern_set.db_type, mat.as_str().to_string()));
                }
            }
        }
        None
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
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();

        // Phase 1: Error-based detection
        let error_payloads = payload_selector.select(ActiveVulnType::SqlInjectionError);
        for payload_def in &error_payloads {
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

            if let Some((db_type, matched)) = self.identify_sql_error(&body) {
                // DIFFERENTIAL ANALYSIS: False Positive Check
                // If the baseline inherently has this "error", ignore it!
                let is_false_positive = baseline
                    .map(|b| b.contains_indicator(&matched))
                    .unwrap_or(false);

                if is_false_positive {
                    continue; // Skip, it's just normal page content
                }

                findings.push(ActiveVulnFinding {
                    id: Uuid::new_v4(),
                    scan_id,
                    timestamp: Utc::now(),
                    vuln_type: ActiveVulnType::SqlInjectionError,
                    target_url: target_url.to_string(),
                    affected_parameter: parameter.to_string(),
                    http_method: "GET".to_string(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: format!("GET {} HTTP/1.1", test_url),
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: elapsed,
                        matched_indicator: format!("SQL error from {} DB: {}", db_type, matched),
                        additional_notes: vec![
                            format!("Detected database: {}", db_type),
                            "SQL error message exposed in response".to_string(),
                        ],
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

        // Phase 1.5: Boolean-Based Blind SQLi Verification (Differential Analysis)
        if findings.is_empty() {
            if let Some(base) = baseline {
                // Generate True and False payload URLs
                let true_payload = "1' AND 1=1-- -";
                let false_payload = "1' AND 1=2-- -";

                let true_url = format!(
                    "{}?{}={}",
                    target_url,
                    parameter,
                    urlencoding::encode(true_payload)
                );
                let false_url = format!(
                    "{}?{}={}",
                    target_url,
                    parameter,
                    urlencoding::encode(false_payload)
                );

                // Send True Payload
                let mut true_req = self.client.get(&true_url);
                if payload_selector.is_waf_bypass_enabled() {
                    true_req =
                    crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(
                        true_req,
                    );
                }
                if let Ok(true_resp) = true_req.send().await {
                    let t_status = true_resp.status().as_u16();
                    let t_body = true_resp.text().await.unwrap_or_default();

                    // Send False Payload
                    let mut false_req = self.client.get(&false_url);
                    if payload_selector.is_waf_bypass_enabled() {
                        false_req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(false_req);
                    }
                    if let Ok(false_resp) = false_req.send().await {
                        let f_status = false_resp.status().as_u16();
                        let f_body = false_resp.text().await.unwrap_or_default();

                        // If TRUE matches baseline AND FALSE diverges from baseline -> 100% SQLi
                        let true_is_similar = !base.is_significantly_different(t_status, &t_body);
                        let false_is_different = base.is_significantly_different(f_status, &f_body);

                        if true_is_similar && false_is_different {
                            findings.push(ActiveVulnFinding {
                                id: Uuid::new_v4(),
                                scan_id,
                                timestamp: Utc::now(),
                                vuln_type: ActiveVulnType::SqlInjectionBlindBoolean,
                                target_url: target_url.to_string(),
                                affected_parameter: parameter.to_string(),
                                http_method: "GET".to_string(),
                                payload_used: true_payload.to_string(),
                                evidence: ActiveVulnEvidence {
                                    request_raw: format!(
                                        "GET {} HTTP/1.1\n\nGET {} HTTP/1.1",
                                        true_url, false_url
                                    ),
                                    response_raw: format!(
                                        "True length: {}, False length: {}, Baseline: {}",
                                        t_body.len(),
                                        f_body.len(),
                                        base.content_length
                                    ),
                                    response_time_ms: 0,
                                    matched_indicator: "Differential Content Analysis".to_string(),
                                    additional_notes: vec![
                                        "Boolean Blind SQL Injection verified via True/False logic"
                                            .to_string(),
                                    ],
                                },
                                severity: SeverityLevel::Critical,
                                confidence: ConfidenceLevel::Certain,
                                exploitability: ExploitabilityLevel::Actionable,
                                poc_generated: false,
                                poc_id: None,
                                verified: true,
                                false_positive: false,
                            });
                        }
                    }
                }
            }
        }

        // Phase 2: Time-based blind detection
        if findings.is_empty() {
            let time_payloads = payload_selector.select(ActiveVulnType::SqlInjectionBlindTime);
            for payload_def in &time_payloads {
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
                    req = crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(req);
                }
                let resp = req.send().await.map_err(|e| e.to_string())?;

                let status = resp.status().as_u16();
                waf_monitor.register_response(target_url, status);
                if waf_monitor.is_waf_detected(target_url) {
                    tokio::time::sleep(std::time::Duration::from_millis(rate_limit_ms * 2)).await;
                }

                let elapsed = start.elapsed().as_millis() as u64;

                // If response takes >= 4500ms (expected ~5000ms), it's likely time-based blind
                if elapsed >= 4500 {
                    findings.push(ActiveVulnFinding {
                        id: Uuid::new_v4(),
                        scan_id,
                        timestamp: Utc::now(),
                        vuln_type: ActiveVulnType::SqlInjectionBlindTime,
                        target_url: target_url.to_string(),
                        affected_parameter: parameter.to_string(),
                        http_method: "GET".to_string(),
                        payload_used: payload_def.payload.clone(),
                        evidence: ActiveVulnEvidence {
                            request_raw: format!("GET {} HTTP/1.1", test_url),
                            response_raw: format!("Response delayed: {}ms", elapsed),
                            response_time_ms: elapsed,
                            matched_indicator: format!(
                                "Time delay: {}ms (expected ~5000ms)",
                                elapsed
                            ),
                            additional_notes: vec![
                                "Time-based blind SQL injection detected".to_string(),
                                format!("Response took {}ms vs normal <1s", elapsed),
                            ],
                        },
                        severity: SeverityLevel::Critical,
                        confidence: ConfidenceLevel::Firm,
                        exploitability: ExploitabilityLevel::Conditional,
                        poc_generated: false,
                        poc_id: None,
                        verified: false,
                        false_positive: false,
                    });
                    break;
                }
            }
        }

        Ok(findings)
    }
}
