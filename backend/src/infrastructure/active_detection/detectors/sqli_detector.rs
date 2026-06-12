use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use uuid::Uuid;

use super::VulnDetector;
use crate::domain::active_vuln::*;
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};

struct SqlErrorPattern {
    db_type: &'static str,
    patterns: Vec<Regex>,
}

pub struct SqliDetector {
    client: Client,

    error_patterns: Vec<SqlErrorPattern>,
}

impl SqliDetector {
    pub fn new(client: Client) -> Self {
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
        endpoint_ctx: Option<&crate::domain::fuzzing::EndpointContext>,
        insertion_point: Option<&crate::domain::fuzzing::InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut findings = Vec::new();

        // Phase 1: Error-based detection
        let error_payloads = payload_selector.select(ActiveVulnType::SqlInjectionError);
        for payload_def in &error_payloads {
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
                    target_url: payload_response.request_url.clone(),
                    affected_parameter: parameter.to_string(),
                    http_method: payload_response.http_method.clone(),
                    payload_used: payload_def.payload.clone(),
                    evidence: ActiveVulnEvidence {
                        request_raw: payload_response.request_raw,
                        response_raw: body.chars().take(2000).collect(),
                        response_time_ms: payload_response.response_time_ms,
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
                let true_payload = "1' AND 1=1-- -";
                let false_payload = "1' AND 1=2-- -";

                let true_response = super::send_payload_request(
                    &self.client,
                    target_url,
                    parameter,
                    true_payload,
                    endpoint_ctx,
                    insertion_point,
                    payload_selector.is_waf_bypass_enabled(),
                )
                .await;
                let false_response = super::send_payload_request(
                    &self.client,
                    target_url,
                    parameter,
                    false_payload,
                    endpoint_ctx,
                    insertion_point,
                    payload_selector.is_waf_bypass_enabled(),
                )
                .await;

                if let (Ok(true_resp), Ok(false_resp)) = (true_response, false_response) {
                    let t_status = true_resp.status_code;
                    let f_status = false_resp.status_code;
                    let t_body = true_resp.response_body.clone();
                    let f_body = false_resp.response_body.clone();

                    // If TRUE matches baseline AND FALSE diverges from baseline -> 100% SQLi
                    let true_is_similar = !base.is_significantly_different(t_status, &t_body);
                    let false_is_different = base.is_significantly_different(f_status, &f_body);

                    if true_is_similar && false_is_different {
                        findings.push(ActiveVulnFinding {
                            id: Uuid::new_v4(),
                            scan_id,
                            timestamp: Utc::now(),
                            vuln_type: ActiveVulnType::SqlInjectionBlindBoolean,
                            target_url: true_resp.request_url.clone(),
                            affected_parameter: parameter.to_string(),
                            http_method: true_resp.http_method.clone(),
                            payload_used: true_payload.to_string(),
                            evidence: ActiveVulnEvidence {
                                request_raw: format!(
                                    "{}\n\n{}",
                                    true_resp.request_raw, false_resp.request_raw
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

        // Phase 2: Time-based blind detection
        if findings.is_empty() {
            let time_payloads = payload_selector.select(ActiveVulnType::SqlInjectionBlindTime);
            for payload_def in &time_payloads {
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

                // If response takes >= 4500ms (expected ~5000ms), it's likely time-based blind
                if payload_response.response_time_ms >= 4500 {
                    findings.push(ActiveVulnFinding {
                        id: Uuid::new_v4(),
                        scan_id,
                        timestamp: Utc::now(),
                        vuln_type: ActiveVulnType::SqlInjectionBlindTime,
                        target_url: payload_response.request_url.clone(),
                        affected_parameter: parameter.to_string(),
                        http_method: payload_response.http_method.clone(),
                        payload_used: payload_def.payload.clone(),
                        evidence: ActiveVulnEvidence {
                            request_raw: payload_response.request_raw,
                            response_raw: format!(
                                "Response delayed: {}ms",
                                payload_response.response_time_ms
                            ),
                            response_time_ms: payload_response.response_time_ms,
                            matched_indicator: format!(
                                "Time delay: {}ms (expected ~5000ms)",
                                payload_response.response_time_ms
                            ),
                            additional_notes: vec![
                                "Time-based blind SQL injection detected".to_string(),
                                format!(
                                    "Response took {}ms vs normal <1s",
                                    payload_response.response_time_ms
                                ),
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
