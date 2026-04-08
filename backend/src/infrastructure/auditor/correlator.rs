use crate::domain::entities::*;
use uuid::Uuid;

pub struct CorrelationEngine;

impl CorrelationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn correlate(&self, findings: &[SecurityAuditFinding]) -> Vec<CorrelationResult> {
        let mut results = Vec::new();

        // Strategy 1: Compound Risk – Multiple missing headers on same target
        self.detect_compound_header_weaknesses(findings, &mut results);

        // Strategy 2: Supporting Signal – Cookie weakness + Auth bypass on same target
        self.detect_cookie_auth_support(findings, &mut results);

        // Strategy 3: Repeated Surface – Same category across multiple modules
        self.detect_repeated_surface(findings, &mut results);

        // Strategy 4: Contextual Link – Sensitive endpoint + dangerous method
        self.detect_endpoint_method_link(findings, &mut results);

        // Strategy 5: Duplicate Signal – Near-identical summaries across modules
        self.detect_duplicate_signals(findings, &mut results);

        // Strategy 6: CSP + Inline Scripts + Input Exposure
        self.detect_csp_inline_input(findings, &mut results);

        // Strategy 7: Permissive CORS + Configured Credentials on Sensitive path
        self.detect_cors_credentials(findings, &mut results);

        results
    }

    /// Multiple SecurityMisconfiguration findings (headers) on the same target
    fn detect_compound_header_weaknesses(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let header_findings: Vec<&SecurityAuditFinding> = findings
            .iter()
            .filter(|f| {
                f.category == FindingCategory::SecurityMisconfiguration
                    && f.summary.to_lowercase().contains("header")
            })
            .collect();

        if header_findings.len() >= 2 {
            // Gate: at least one must be Medium+ severity
            let has_meaningful = header_findings
                .iter()
                .any(|f| matches!(f.severity, SeverityLevel::Medium | SeverityLevel::High | SeverityLevel::Critical));

            if !has_meaningful {
                return;
            }

            let group_id = Uuid::new_v4().to_string();
            let links: Vec<CorrelationLink> = header_findings
                .iter()
                .map(|f| CorrelationLink {
                    finding_id: f.id.clone(),
                    relationship_note: format!("Header weakness: {}", f.summary),
                })
                .collect();

            let has_dynamic_context = header_findings.iter().any(|f| {
                let combined = format!("{} {}", f.summary, f.technical_details).to_lowercase();
                combined.contains("input") || combined.contains("script") || combined.contains("login") || combined.contains("admin") || combined.contains("api")
            });
            let is_hygiene_gap = !has_dynamic_context;

            results.push(CorrelationResult {
                group_id,
                core_target: header_findings[0].target_identifier.clone(),
                correlation_type: CorrelationType::CompoundRisk,
                confidence: ConfidenceLevel::Firm,
                summary: format!(
                    "Multiple header weaknesses detected on the same endpoint ({} findings)",
                    header_findings.len()
                ),
                reason: CorrelationReason {
                    code: "COMPOUND_HEADERS".into(),
                    explanation: "Several security headers are missing or misconfigured on the same target. They serve as a hygiene gap unless a dynamic attack surface is confirmed.".into(),
                },
                linked_findings: links,
                is_hygiene_gap,
            });
        }
    }

    /// Cookie weakness combined with authentication-related exposure
    fn detect_cookie_auth_support(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let cookie_findings: Vec<&SecurityAuditFinding> = findings
            .iter()
            .filter(|f| {
                f.summary.to_lowercase().contains("cookie")
                    || f.summary.to_lowercase().contains("httponly")
                    || f.summary.to_lowercase().contains("samesite")
            })
            .collect();

        let auth_findings: Vec<&SecurityAuditFinding> = findings
            .iter()
            .filter(|f| f.category == FindingCategory::AuthenticationBypass)
            .collect();

        if !cookie_findings.is_empty() && !auth_findings.is_empty() {
            // Gate: auth finding must be at least Tentative confidence
            let auth_valid = auth_findings.iter().any(|f| {
                matches!(
                    f.confidence,
                    ConfidenceLevel::Tentative | ConfidenceLevel::Firm | ConfidenceLevel::Certain
                )
            });

            if !auth_valid {
                return;
            }

            let group_id = Uuid::new_v4().to_string();
            let mut links: Vec<CorrelationLink> = Vec::new();

            for f in &cookie_findings {
                links.push(CorrelationLink {
                    finding_id: f.id.clone(),
                    relationship_note: format!("Cookie weakness: {}", f.summary),
                });
            }
            for f in &auth_findings {
                links.push(CorrelationLink {
                    finding_id: f.id.clone(),
                    relationship_note: format!("Auth exposure: {}", f.summary),
                });
            }

            results.push(CorrelationResult {
                group_id,
                core_target: cookie_findings[0].target_identifier.clone(),
                correlation_type: CorrelationType::SupportingSignal,
                confidence: ConfidenceLevel::Firm,
                summary: "Authentication-related weakness supported by cookie evidence".into(),
                reason: CorrelationReason {
                    code: "COOKIE_AUTH_SUPPORT".into(),
                    explanation: "Weak cookie configuration combined with authentication bypass indicators increases exploitation likelihood.".into(),
                },
                linked_findings: links,
                is_hygiene_gap: false,
            });
        }
    }

    /// Same category reported from 2+ different source modules
    fn detect_repeated_surface(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let categories: Vec<&FindingCategory> = findings.iter().map(|f| &f.category).collect();
        let unique_categories: std::collections::HashSet<String> =
            categories.iter().map(|c| format!("{:?}", c)).collect();

        for cat_str in &unique_categories {
            let cat_findings: Vec<&SecurityAuditFinding> = findings
                .iter()
                .filter(|f| format!("{:?}", f.category) == *cat_str)
                .collect();

            // Need findings from at least 2 different modules
            let modules: std::collections::HashSet<String> = cat_findings
                .iter()
                .map(|f| format!("{:?}", f.source_module))
                .collect();

            if modules.len() >= 2 {
                // Gate: suppress if all findings are Informational
                let has_substance = cat_findings
                    .iter()
                    .any(|f| !matches!(f.severity, SeverityLevel::Informational));

                if !has_substance {
                    continue;
                }

                let group_id = Uuid::new_v4().to_string();
                let links: Vec<CorrelationLink> = cat_findings
                    .iter()
                    .map(|f| CorrelationLink {
                        finding_id: f.id.clone(),
                        relationship_note: format!(
                            "Module {:?}: {}",
                            f.source_module, f.summary
                        ),
                    })
                    .collect();

                let all_low = cat_findings.iter().all(|f| f.severity == SeverityLevel::Low);

                results.push(CorrelationResult {
                    group_id,
                    core_target: cat_findings[0].target_identifier.clone(),
                    correlation_type: CorrelationType::RepeatedSurface,
                    confidence: ConfidenceLevel::Firm,
                    summary: format!(
                        "Category '{}' confirmed by multiple modules ({} sources)",
                        cat_str,
                        modules.len()
                    ),
                    reason: CorrelationReason {
                        code: "REPEATED_SURFACE".into(),
                        explanation: format!(
                            "Findings of category '{}' were independently reported by {} different modules, reinforcing the observation.",
                            cat_str,
                            modules.len()
                        ),
                    },
                    linked_findings: links,
                    is_hygiene_gap: all_low,
                });
            }
        }
    }

    /// Sensitive endpoint + unusual HTTP method
    fn detect_endpoint_method_link(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let sensitive_endpoints: Vec<&SecurityAuditFinding> = findings
            .iter()
            .filter(|f| {
                f.category == FindingCategory::AuthenticationBypass
                    || f.category == FindingCategory::SuspiciousEndpoint
            })
            .filter(|f| f.affected_path_or_endpoint.is_some())
            .collect();

        let dangerous_methods = ["DELETE", "PUT", "PATCH"];

        for endpoint_finding in &sensitive_endpoints {
            if let Some(ref method) = endpoint_finding.method {
                let method_upper = method.to_uppercase();
                if dangerous_methods.contains(&method_upper.as_str()) {
                    let group_id = Uuid::new_v4().to_string();
                    results.push(CorrelationResult {
                        group_id,
                        core_target: endpoint_finding.target_identifier.clone(),
                        correlation_type: CorrelationType::ContextualLink,
                        confidence: ConfidenceLevel::Tentative,
                        summary: format!(
                            "Sensitive endpoint '{}' accessible via dangerous method '{}'",
                            endpoint_finding
                                .affected_path_or_endpoint
                                .as_deref()
                                .unwrap_or("unknown"),
                            method
                        ),
                        reason: CorrelationReason {
                            code: "ENDPOINT_METHOD_LINK".into(),
                            explanation: "A sensitive or authentication-bypassed endpoint is accessible via a destructive HTTP method, increasing risk.".into(),
                        },
                        linked_findings: vec![CorrelationLink {
                            finding_id: endpoint_finding.id.clone(),
                            relationship_note: format!(
                                "Endpoint {} with method {}",
                                endpoint_finding
                                    .affected_path_or_endpoint
                                    .as_deref()
                                    .unwrap_or("unknown"),
                                method
                            ),
                        }],
                        is_hygiene_gap: false,
                    });
                }
            }
        }
    }

    /// Near-identical summaries across different source modules
    fn detect_duplicate_signals(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let len = findings.len();
        let mut seen_pairs: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();

        for i in 0..len {
            for j in (i + 1)..len {
                let a = &findings[i];
                let b = &findings[j];

                // Must be from different modules
                if a.source_module == b.source_module {
                    continue;
                }

                // Gate: both must be at least Low confidence
                if matches!(a.confidence, ConfidenceLevel::Low)
                    && matches!(b.confidence, ConfidenceLevel::Low)
                {
                    continue;
                }

                // Check summary similarity (simple keyword overlap)
                let a_lower = a.summary.to_lowercase();
                let b_lower = b.summary.to_lowercase();
                let a_words: std::collections::HashSet<&str> =
                    a_lower.split_whitespace().collect();
                let b_words: std::collections::HashSet<&str> =
                    b_lower.split_whitespace().collect();
                let intersection = a_words.intersection(&b_words).count();
                let min_len = a_words.len().min(b_words.len());

                if min_len > 0 && (intersection as f64 / min_len as f64) > 0.6 {
                    let pair_key = if a.id < b.id {
                        (a.id.clone(), b.id.clone())
                    } else {
                        (b.id.clone(), a.id.clone())
                    };

                    if seen_pairs.contains(&pair_key) {
                        continue;
                    }
                    seen_pairs.insert(pair_key);

                    let group_id = Uuid::new_v4().to_string();
                    results.push(CorrelationResult {
                        group_id,
                        core_target: a.target_identifier.clone(),
                        correlation_type: CorrelationType::DuplicateSignal,
                        confidence: ConfidenceLevel::Tentative,
                        summary: format!(
                            "Similar finding reported by {:?} and {:?}: '{}'",
                            a.source_module, b.source_module, a.summary
                        ),
                        reason: CorrelationReason {
                            code: "DUPLICATE_SIGNAL".into(),
                            explanation: "Two modules independently produced findings with very similar summaries, suggesting overlapping detection.".into(),
                        },
                        linked_findings: vec![
                            CorrelationLink {
                                finding_id: a.id.clone(),
                                relationship_note: format!("From {:?}: {}", a.source_module, a.summary),
                            },
                            CorrelationLink {
                                finding_id: b.id.clone(),
                                relationship_note: format!("From {:?}: {}", b.source_module, b.summary),
                            },
                        ],
                        is_hygiene_gap: false,
                    });
                }
            }
        }
    }

    /// Strict Correlation: Missing CSP + Inline Scripts + Input Exposure
    fn detect_csp_inline_input(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let csp_findings: Vec<&SecurityAuditFinding> = findings.iter()
            .filter(|f| f.summary.to_lowercase().contains("content-security-policy") || f.summary.to_lowercase().contains("csp"))
            .collect();
            
        let inline_findings: Vec<&SecurityAuditFinding> = findings.iter()
            .filter(|f| f.summary.to_lowercase().contains("inline script") || f.technical_details.to_lowercase().contains("inline script"))
            .collect();
            
        let input_findings: Vec<&SecurityAuditFinding> = findings.iter()
            .filter(|f| f.summary.to_lowercase().contains("input") || f.technical_details.to_lowercase().contains("reflected"))
            .collect();

        if !csp_findings.is_empty() && !inline_findings.is_empty() && !input_findings.is_empty() {
            let group_id = Uuid::new_v4().to_string();
            let mut links = Vec::new();
            if let Some(f) = csp_findings.first() { links.push(CorrelationLink { finding_id: f.id.clone(), relationship_note: "Missing CSP".into() }); }
            if let Some(f) = inline_findings.first() { links.push(CorrelationLink { finding_id: f.id.clone(), relationship_note: "Inline Scripts".into() }); }
            if let Some(f) = input_findings.first() { links.push(CorrelationLink { finding_id: f.id.clone(), relationship_note: "Input Exposure".into() }); }

            results.push(CorrelationResult {
                group_id,
                core_target: csp_findings[0].target_identifier.clone(),
                correlation_type: CorrelationType::CompoundRisk,
                confidence: ConfidenceLevel::Certain,
                summary: "High Exploitability: Missing CSP combined with inline scripts and user input exposure".into(),
                reason: CorrelationReason {
                    code: "CSP_XSS_CHAIN".into(),
                    explanation: "The absence of Content-Security-Policy combined with reflected user input and inline scripts creates a highly exploitable Cross-Site Scripting (XSS) pathway.".into(),
                },
                linked_findings: links,
                is_hygiene_gap: false,
            });
        }
    }

    /// Strict Correlation: Permissive CORS + Credentialed Behavior on Sensitive Path
    fn detect_cors_credentials(
        &self,
        findings: &[SecurityAuditFinding],
        results: &mut Vec<CorrelationResult>,
    ) {
        let cors_findings: Vec<&SecurityAuditFinding> = findings.iter()
            .filter(|f| f.summary.to_lowercase().contains("cors") || f.summary.to_lowercase().contains("access-control"))
            .collect();

        let cred_findings: Vec<&SecurityAuditFinding> = findings.iter()
            .filter(|f| {
                let combined = format!("{} {}", f.summary, f.technical_details).to_lowercase();
                combined.contains("credential") || combined.contains("allow-credentials") || combined.contains("login") || combined.contains("auth")
            })
            .collect();

        if !cors_findings.is_empty() && !cred_findings.is_empty() && cors_findings.iter().any(|f| !cred_findings.iter().any(|c| c.id == f.id)) {
            let group_id = Uuid::new_v4().to_string();
            let mut links = Vec::new();
            if let Some(f) = cors_findings.first() { links.push(CorrelationLink { finding_id: f.id.clone(), relationship_note: "Permissive CORS".into() }); }
            if let Some(f) = cred_findings.first() { links.push(CorrelationLink { finding_id: f.id.clone(), relationship_note: "Credentialed Endpoint".into() }); }

            results.push(CorrelationResult {
                group_id,
                core_target: cors_findings[0].target_identifier.clone(),
                correlation_type: CorrelationType::CompoundRisk,
                confidence: ConfidenceLevel::Certain,
                summary: "High Risk: Permissive CORS configuration affecting credentialed endpoints".into(),
                reason: CorrelationReason {
                    code: "CORS_CREDENTIAL_HIJACK".into(),
                    explanation: "Permissive Cross-Origin Resource Sharing on endpoints that accept or rely on credentials opens the door for cross-site attack exfiltration.".into(),
                },
                linked_findings: links,
                is_hygiene_gap: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_finding(category: FindingCategory, summary: &str, details: &str, severity: SeverityLevel) -> SecurityAuditFinding {
        SecurityAuditFinding {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            target_identifier: "https://example.com".to_string(),
            affected_path_or_endpoint: None,
            protocol: None,
            method: None,
            category,
            severity,
            confidence: ConfidenceLevel::Tentative,
            status: FindingStatus::Open,
            summary: summary.to_string(),
            technical_details: details.to_string(),
            source_module: SourceModule::WebScanner,
            evidence: Vec::new(),
            raw_reference: None,
            correlation_group_id: None,
            correlation_count: 0,
            correlation_type: None,
            correlation_confidence: None,
            correlation_summary: None,
            correlation_is_hygiene_gap: false,
            related_findings: Vec::new(),
            risk_score: None,
            exploitability: None,
            context_summary: None,
            evidence_weight: None,
            context_assessment: None,
            certainty: None,
        }
    }

    #[test]
    fn test_compound_headers_hygiene_gap() {
        let correlator = CorrelationEngine::new();
        let findings = vec![
            mock_finding(FindingCategory::SecurityMisconfiguration, "Missing CSP Header", "No dynamic context here", SeverityLevel::Medium),
            mock_finding(FindingCategory::SecurityMisconfiguration, "Missing HSTS Header", "Just a static page", SeverityLevel::Medium),
        ];
        
        let mut results = correlator.correlate(&findings);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_hygiene_gap);
    }

    #[test]
    fn test_compound_headers_with_dynamic_input() {
        let correlator = CorrelationEngine::new();
        let findings = vec![
            mock_finding(FindingCategory::SecurityMisconfiguration, "Missing CSP Header", "Reflected input affects page", SeverityLevel::High),
            mock_finding(FindingCategory::SecurityMisconfiguration, "Missing X-Frame-Options Header", "Static", SeverityLevel::Medium),
        ];

        let mut results = correlator.correlate(&findings);
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_hygiene_gap);
    }

    #[test]
    fn test_strict_correlation_csp_xss() {
        let correlator = CorrelationEngine::new();
        let findings = vec![
            mock_finding(FindingCategory::SecurityMisconfiguration, "Content-Security-Policy disabled", "", SeverityLevel::Medium),
            mock_finding(FindingCategory::InformationDisclosure, "Found inline script block", "", SeverityLevel::Informational),
            mock_finding(FindingCategory::SuspiciousEndpoint, "Reflected User Input detected", "input exposed", SeverityLevel::Medium),
        ];

        let mut results = correlator.correlate(&findings);
        // Should trigger both Duplicate/Overlaps if any, but specifically CSP_XSS
        let xss_chain = results.iter().find(|c| c.reason.code == "CSP_XSS_CHAIN");
        assert!(xss_chain.is_some());
        assert!(!xss_chain.unwrap().is_hygiene_gap);
    }
}
