use crate::domain::entities::*;
use crate::domain::repositories::SecurityAuditorRepository;
use async_trait::async_trait;
use reqwest::Client;
use url::Url;
use super::normalizer::{FindingNormalizer, WebScannerNormalizer, ServerInvestigatorNormalizer, ApiDiscovererNormalizer};

pub struct HttpSecurityAuditor {
    client: Client,
}

impl HttpSecurityAuditor {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .user_agent("LimmaSecurityAuditor/3.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl SecurityAuditorRepository for HttpSecurityAuditor {
    async fn audit(&self, url_str: &str) -> Result<SecurityReport, String> {
        let resp = self.client.get(url_str).send().await.map_err(|e| e.to_string())?;
        let headers = resp.headers();

        let mut score: i32 = 100; // Allow it to dip slightly if absolutely terrible but we'll cap at 0
        let mut missing_headers = Vec::new();
        let mut recommendations = Vec::new();

        // 1. Extended Header Validation
        let essential_headers = vec![
            ("content-security-policy", "CSP prevents XSS and data injection attacks.", 15),
            ("strict-transport-security", "HSTS forces encrypted connections (HTTPS).", 15),
            ("x-frame-options", "X-Frame-Options prevents Clickjacking.", 10),
            ("x-content-type-options", "Prevents MIME sniffing vulnerabilities.", 5),
            ("referrer-policy", "Controls leakage of referrer information to external sites.", 5),
            ("permissions-policy", "Restricts which browser features/APIs can be used.", 5),
        ];

        for (header_key, reason, penalty) in essential_headers {
            if !headers.contains_key(header_key) {
                score -= penalty;
                missing_headers.push(header_key.to_string());
                recommendations.push(reason.to_string());
            }
        }

        // 2. Cross-Origin Resource Sharing (CORS) check
        if let Some(cors_header) = headers.get("access-control-allow-origin") {
            if cors_header.to_str().unwrap_or("") == "*" {
                score -= 10;
                recommendations.push("CORS policy allows all origins ('*'). This can be highly dangerous for API endpoints.".to_string());
            }
        }

        // 3. Set-Cookie Security Checks
        if let Some(cookie_header) = headers.get("set-cookie") {
            let cookie_str = cookie_header.to_str().unwrap_or("").to_lowercase();
            if !cookie_str.contains("secure") {
                score -= 10;
                recommendations.push("Cookies are missing the 'Secure' flag, transmitting them in plaintext over HTTP is possible.".to_string());
            }
            if !cookie_str.contains("httponly") {
                score -= 15;
                recommendations.push("Cookies are missing the 'HttpOnly' flag, making them accessible to XSS JavaScript payloads.".to_string());
            }
            if !cookie_str.contains("samesite") {
                score -= 5;
                recommendations.push("Cookies do not have a 'SameSite' attribute, increasing CSRF risk.".to_string());
            }
        }

        // 4. File exposure checks (.well-known/security.txt and robots.txt)
        let mut disallowed = Vec::new();
        if let Ok(u) = Url::parse(url_str) {
            let base = format!("{}://{}", u.scheme(), u.host_str().unwrap_or(""));
            
            // Check security.txt
            let mut security_txt_found = false;
            let security_url = format!("{}/.well-known/security.txt", base);
            if let Ok(sec_resp) = self.client.get(&security_url).send().await {
                if sec_resp.status().is_success() {
                    security_txt_found = true;
                }
            }
            if !security_txt_found {
                score -= 5;
                recommendations.push("No '/.well-known/security.txt' was found. Good practice is to provide a contact for security researchers.".to_string());
            }

            // Check robots.txt
            let robot_url = format!("{}/robots.txt", base);
            if let Ok(robot_resp) = self.client.get(&robot_url).send().await {
                if robot_resp.status().is_success() {
                    let robot_body = robot_resp.text().await.unwrap_or_default();
                    let suspicious_keywords = vec!["admin", "login", "wp-", "config", ".env", "backup", "db"];
                    let mut sensitive_leaks = 0;

                    for line in robot_body.lines() {
                        let line_lower = line.to_lowercase();
                        if line_lower.starts_with("disallow:") {
                            let path = line.replace("Disallow:", "").replace("disallow:", "").trim().to_string();
                            disallowed.push(path.clone());
                            
                            for kw in &suspicious_keywords {
                                if path.contains(kw) {
                                    sensitive_leaks += 1;
                                }
                            }
                        }
                    }

                    if sensitive_leaks > 0 {
                        score -= 20;
                        recommendations.push(format!("CRITICAL: Found {} sensitive paths stored in robots.txt which attackers actively scrape.", sensitive_leaks));
                    }
                }
            }
        }

        if score < 0 {
            score = 0;
        }

        Ok(SecurityReport {
            url: url_str.to_string(),
            security_score: score as u8,
            missing_headers,
            robot_rules_disallowed: disallowed,
            recommendations,
        })
    }

    async fn normalize_all(
        &self,
        target: &str,
        web_scan: &WebScanResult,
        server_info: &ServerInfo,
        api_discovery: &ApiDiscoveryResult,
    ) -> Result<NormalizedAuditReport, String> {
        let mut all_findings: Vec<SecurityAuditFinding> = Vec::new();
        let mut log = Vec::new();
        
        log.push(format!("[Normalizer] Started normalization for target: {}", target));
        
        let web_normalizer = WebScannerNormalizer;
        let mut web_findings = web_normalizer.normalize(target, web_scan);
        log.push(format!("[Normalizer] Extracted {} findings from Web Scanner.", web_findings.len()));
        all_findings.append(&mut web_findings);
        
        let server_normalizer = ServerInvestigatorNormalizer;
        let mut server_findings = server_normalizer.normalize(target, server_info);
        log.push(format!("[Normalizer] Extracted {} findings from Server Investigator.", server_findings.len()));
        all_findings.append(&mut server_findings);
        
        let api_normalizer = ApiDiscovererNormalizer;
        let mut api_findings = api_normalizer.normalize(target, api_discovery);
        log.push(format!("[Normalizer] Extracted {} findings from API Discoverer.", api_findings.len()));
        all_findings.append(&mut api_findings);
        let total_findings_raw = all_findings.len();
        log.push("[Normalizer] Initializing False-Positive Mitigation Heuristics...".to_string());
        
        let mut valid_findings = Vec::new();
        let mut rejected = 0;
        
        for mut f in all_findings {
            let mut is_fp = false;
            let mut fp_reason = String::new();
            
            // Heuristic 1: Low-confidence + Low-severity is almost always useless noise.
            if (f.confidence == ConfidenceLevel::Low || f.confidence == ConfidenceLevel::Tentative) 
               && (f.severity == SeverityLevel::Low || f.severity == SeverityLevel::Informational) {
                is_fp = true;
                fp_reason = "Low confidence combined with negligible severity".to_string();
            }
            
            // Heuristic 2: Information Disclosure without any concrete evidence and tentative confidence
            if f.category == FindingCategory::InformationDisclosure 
               && f.evidence.is_empty() 
               && f.confidence != ConfidenceLevel::Certain {
                is_fp = true;
                fp_reason = "Information Disclosure claim lacking concrete evidence".to_string();
            }

            // Heuristic 3: Suspicious Endpoint from ApiDiscoverer with Low confidence
            if f.category == FindingCategory::SuspiciousEndpoint 
               && f.confidence == ConfidenceLevel::Low {
                is_fp = true;
                fp_reason = "Suspicious endpoint match was too generic (Low confidence)".to_string();
            }

            // Heuristic 4: Duplicate structural checks (optional, simplistic deduplication)
            // Skipping full dedup here since correlator handles it, but we can drop literal duplicates
            
            if is_fp {
                rejected += 1;
                // Trace block for deep debugging (invisible to UI unless requested)
                tracing::debug!("FP Filter triggered -> Dropped finding: {} | Reason: {}", f.summary, fp_reason);
            } else {
                f.status = FindingStatus::Open;
                valid_findings.push(f);
            }
        }
        
        let mut all_findings = valid_findings;
        let total_findings = total_findings_raw;
        let accepted = all_findings.len();
        
        log.push(format!("[Normalizer] FP Mitigation Complete. Filtered out {} false positive anomalies.", rejected));
        log.push(format!("[Normalizer] Proceeding with {} verified findings.", accepted));

        // Let's run Phase 2 Rule Engine
        log.push(format!("[RuleEngine] Starting definition enforcement phase..."));
        let engine = super::engine::RuleEngine::new();
        let rule_results = engine.evaluate(&all_findings);
        let matched_count = rule_results.iter().filter(|r| r.outcome == RuleOutcome::Matched).count();
        let partially_matched_count = rule_results.iter().filter(|r| r.outcome == RuleOutcome::PartiallyMatched).count();
        
        log.push(format!("[RuleEngine] Evaluated {} findings against {} rules.", all_findings.len(), engine.rules.len()));
        log.push(format!("[RuleEngine] Matched: {} rules. Partially Matched: {} rules.", matched_count, partially_matched_count));

        // Phase 3: Correlation Engine
        log.push("[Correlator] Starting correlation analysis...".to_string());
        let correlator = super::correlator::CorrelationEngine::new();
        let correlations = correlator.correlate(&all_findings);
        log.push(format!("[Correlator] Analyzed {} findings, produced {} correlation groups.", all_findings.len(), correlations.len()));

        // --- Phase 3.1: Reverse Mapping Correlations into Findings ---
        log.push("[Correlator] Mapping multi-signal clusters back into isolated findings...".to_string());
        let mut correlated_finding_count = 0;
        
        for corr in &correlations {
            let group_id = corr.group_id.clone();
            let count = corr.linked_findings.len();
            let c_type = corr.correlation_type.clone();
            let c_conf = corr.confidence.clone();
            let c_sum = corr.summary.clone();
            let c_hygiene = corr.is_hygiene_gap;
            
            // Build the reference list for all siblings
            let mut siblings: Vec<CorrelatedFindingReference> = Vec::new();
            for link in &corr.linked_findings {
                if let Some(f) = all_findings.iter().find(|x| x.id == link.finding_id) {
                    siblings.push(CorrelatedFindingReference {
                        id: f.id.clone(),
                        category: f.category.clone(),
                        severity: f.severity.clone(),
                        short_summary: f.summary.clone(),
                    });
                }
            }
            
            // Inject into the actual finding structs
            for link in &corr.linked_findings {
                if let Some(f) = all_findings.iter_mut().find(|x| x.id == link.finding_id) {
                    if f.correlation_group_id.is_none() {
                        f.correlation_group_id = Some(group_id.clone());
                        f.correlation_count = count.saturating_sub(1); // Exclude self
                        f.correlation_type = Some(c_type.clone());
                        f.correlation_confidence = Some(c_conf.clone());
                        f.correlation_summary = Some(c_sum.clone());
                        f.correlation_is_hygiene_gap = c_hygiene;
                        
                        f.related_findings = siblings.iter().filter(|s| s.id != f.id).cloned().collect();
                        correlated_finding_count += 1;
                    }
                }
            }
        }
        
        log.push(format!("[Correlator] Rendered {} findings into insight clusters. Isolated findings: {}.", correlated_finding_count, all_findings.len().saturating_sub(correlated_finding_count)));

        // --- Phase 4: Risk Scoring & Prioritization ---
        log.push("[Scorer] Initializing risk scoring engine...".to_string());
        let scorer = super::scorer::RiskScorer::new();
        let scoring_stats = scorer.score_all(&mut all_findings);
        log.push(format!("[Scorer] Scored {} findings. Boosted: {}, Downgraded: {}, Overall Risk: {:.1}",
            scoring_stats.total_scored, scoring_stats.boosted, scoring_stats.downgraded, scoring_stats.overall_risk_score));
        if let Some(ref top) = scoring_stats.top_risk_summary {
            log.push(format!("[Scorer] Top risk: {}", top));
        }
        log.push("[Scorer] Findings sorted by risk score (highest first).".to_string());

        // --- Phase 5: Context-Aware Noise Reduction ---
        log.push("[ContextEval] Starting context-aware evaluation...".to_string());
        let evaluator = super::context_evaluator::ContextEvaluator::new();
        let context_stats = evaluator.evaluate_all(&mut all_findings);
        log.push(format!("[ContextEval] Evaluation complete. Elevated: {}, Downgraded: {}, Suppressed: {}, Unchanged: {}",
            context_stats.elevated, context_stats.downgraded, context_stats.suppressed, context_stats.unchanged));
        log.push("[ContextEval] Findings re-sorted by context-adjusted priority.".to_string());

        // --- Phase 6: Canonical Normalization ---
        log.push("[Canonicalizer] Merging duplicates into Canonical records...".to_string());
        let canonical_engine = super::canonicalizer::CanonicalFindingEngine::new();
        let mut canonical_findings = canonical_engine.canonicalize(&all_findings);
        log.push(format!("[Canonicalizer] Condensed {} raw findings into {} unique canonical findings.", all_findings.len(), canonical_findings.len()));

        // --- Phase 7: Exploitability Engine (Runtime Probes) ---
        log.push("[ExploitabilityEngine] Performing dynamic runtime checks...".to_string());
        let exploitability_engine = super::exploitability::ExploitabilityEngine::new();
        exploitability_engine.evaluate(target, &mut canonical_findings).await;
        log.push("[ExploitabilityEngine] Runtime assessments complete.".to_string());

        // --- Phase 8: Attack Path Correlator ---
        log.push("[AttackPathEngine] Analyzing contextual relationships to detect exploit scenarios...".to_string());
        let attack_path_engine = super::attack_path_correlator::AttackPathEngine::new();
        let mut attack_paths = attack_path_engine.build_paths(&canonical_findings);
        log.push(format!("[AttackPathEngine] Derived {} potential high-level attack paths.", attack_paths.len()));

        // --- Phase 9: Autonomous Verification Engine ---
        log.push("[AutonomousVerificationEngine] Running deep multi-route checks and capturing reproducible traces...".to_string());
        let autonomous_engine = super::autonomous_verification::AutonomousVerificationEngine::new();
        autonomous_engine.verify_all(target, &mut canonical_findings, &mut attack_paths).await;
        log.push("[AutonomousVerificationEngine] Validation state machine concluded successfully.".to_string());

        // --- Learning Feedback Engine Initializer ---
        let learning_engine = super::learning_feedback::LearningFeedbackEngine::new();

        // --- Phase 10: Confidence Calibration Engine ---
        log.push("[ConfidenceCalibrationEngine] Calculating historical empirical reliability to calibrate confidence...".to_string());
        let mut calibration_engine = super::confidence_calibration::ConfidenceCalibrationEngine::new();
        calibration_engine.update_from_scan(&mut canonical_findings, &learning_engine);
        log.push("[ConfidenceCalibrationEngine] Confidence levels calibrated using historical pattern metrics.".to_string());

        // --- Phase 11: Threat Prioritization Engine ---
        log.push("[ThreatPrioritizationEngine] Evaluating real-world impact and calculating final actionability scores...".to_string());
        let prioritization_engine = super::threat_prioritization::ThreatPrioritizationEngine::new();
        prioritization_engine.evaluate_all(&mut canonical_findings, &mut attack_paths, &learning_engine);
        log.push(format!("[ThreatPrioritizationEngine] Successfully ranked {} threats and {} attack paths.", canonical_findings.len(), attack_paths.len()));

        // Compute audit certainty
        let audit_certainty = if all_findings.iter().any(|f| f.correlation_count > 0) {
            Some(CertaintyNote {
                level: CertaintyLevel::Certain,
                reason: "Bulgular çoklu modül tarafından doğrulandı ve korelasyon grupları oluşturuldu".to_string(),
            })
        } else if !all_findings.is_empty() {
            Some(CertaintyNote {
                level: CertaintyLevel::Likely,
                reason: "Bulgular tespit edildi ama çapraz modül doğrulaması yapılamadı".to_string(),
            })
        } else {
            Some(CertaintyNote {
                level: CertaintyLevel::Unknown,
                reason: "Hiçbir güvenlik bulgusu üretilemedi — sonuçlar güvenilir değil".to_string(),
            })
        };

        Ok(NormalizedAuditReport {
            target: target.to_string(),
            timestamp: chrono::Utc::now(),
            total_findings,
            findings: all_findings,
            accepted_findings: accepted,
            rejected_findings: rejected,
            normalization_log: log,
            rule_results,
            correlations,
            canonical_findings,
            attack_paths,
            scoring_stats: Some(scoring_stats),
            context_stats: Some(context_stats),
            audit_certainty,
        })
    }
}
