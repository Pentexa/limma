use crate::domain::entities::*;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

pub struct CanonicalFindingEngine;

impl CanonicalFindingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn canonicalize(&self, findings: &[SecurityAuditFinding]) -> Vec<CanonicalFinding> {
        let mut groups: HashMap<String, Vec<SecurityAuditFinding>> = HashMap::new();

        // 1. Group findings by Canonical Slug
        for finding in findings {
            let slug = self.generate_canonical_slug(finding);
            groups.entry(slug).or_default().push(finding.clone());
        }

        // 2. Reduce groups into Canonical Findings
        let mut canonical_findings = Vec::new();

        for (slug, group_findings) in groups {
            let canonical = self.merge_group(slug, &group_findings);
            canonical_findings.push(canonical);
        }

        // Sort descending by severity and confidence
        canonical_findings.sort_by(|a, b| {
            let sev_a = self.severity_val(&a.severity);
            let sev_b = self.severity_val(&b.severity);
            if sev_a != sev_b {
                sev_b.cmp(&sev_a)
            } else {
                let conf_a = self.confidence_val(&a.confidence);
                let conf_b = self.confidence_val(&b.confidence);
                conf_b.cmp(&conf_a)
            }
        });

        canonical_findings
    }

    fn generate_canonical_slug(&self, finding: &SecurityAuditFinding) -> String {
        let target = &finding.target_identifier;
        let risk_family = format!("{:?}", finding.category).to_lowercase();
        
        let lower_summary = finding.summary.to_lowercase();
        
        // Root Cause extraction
        let root_cause = if lower_summary.contains("strict-transport-security") || lower_summary.contains("hsts") {
            "missing-hsts".to_string()
        } else if lower_summary.contains("content-security-policy") || lower_summary.contains("csp") {
            "missing-csp".to_string()
        } else if lower_summary.contains("x-frame-options") {
            "missing-xframe".to_string()
        } else if lower_summary.contains("cookie") || lower_summary.contains("httponly") || lower_summary.contains("samesite") || lower_summary.contains("secure") {
            "insecure-cookie".to_string()
        } else if lower_summary.contains("cors") || lower_summary.contains("access-control") {
            "permissive-cors".to_string()
        } else if lower_summary.contains("robot") || lower_summary.contains("disallow") {
            "robots-txt-leak".to_string()
        } else if lower_summary.contains("tls") || lower_summary.contains("ssl") || lower_summary.contains("certificate") || lower_summary.contains("cipher") {
            "weak-tls".to_string()
        } else if lower_summary.contains("version") || lower_summary.contains("server") || lower_summary.contains("x-powered-by") {
            "server-banner".to_string()
        } else {
            // sanitize summary to form a slug component
            let sanitized: String = lower_summary.chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect();
            let mut deduplicated = String::new();
            let mut last_char = ' ';
            for c in sanitized.chars() {
                if c == '-' && last_char == '-' { continue; }
                deduplicated.push(c);
                last_char = c;
            }
            deduplicated.trim_matches('-').to_string()
        };

        // Context check to split exploitability contexts (auth/dynamic vs static hygiene)
        let combined_text = format!("{} {}", finding.summary, finding.technical_details).to_lowercase();
        let has_exploit_indicators = combined_text.contains("auth") || combined_text.contains("login") 
            || combined_text.contains("session") || combined_text.contains("credential")
            || finding.category == FindingCategory::SuspiciousEndpoint
            || finding.category == FindingCategory::AuthenticationBypass;

        // E.g. Missing HSTS on public marketing site vs Missing HSTS on login endpoint
        let context_slug = if has_exploit_indicators { "-exploitable-surface" } else { "-standard-hygiene" };

        let protocol_slug = finding.protocol.as_deref().unwrap_or("no-protocol").to_lowercase();

        format!("{}-{}-{}-{}{}", target, protocol_slug, risk_family, root_cause, context_slug)
    }

    fn merge_group(&self, slug: String, group_findings: &[SecurityAuditFinding]) -> CanonicalFinding {
        let title_candidate = &group_findings[0].summary; // Best effort title
        let mut title = self.normalize_title(title_candidate);
        
        let risk_family = group_findings[0].category.clone();
        
        // Modules tracking
        let mut modules_set = HashSet::new();
        let mut routes_set = HashSet::new();
        let mut total_evidence = 0;
        
        let mut max_severity_val = -1;
        let mut max_severity = SeverityLevel::Informational;
        
        let mut max_confidence_val = -1;
        let mut max_confidence = ConfidenceLevel::Low;
        
        // Find best fields
        for f in group_findings {
            modules_set.insert(f.source_module.clone());
            if let Some(r) = &f.affected_path_or_endpoint {
                routes_set.insert(r.clone());
            }
            total_evidence += f.evidence.len();

            let s_val = self.severity_val(&f.severity);
            if s_val > max_severity_val {
                max_severity_val = s_val;
                max_severity = f.severity.clone();
                // Ensure title takes the most severe finding's summary
                title = self.normalize_title(&f.summary);
            }

            let c_val = self.confidence_val(&f.confidence);
            if c_val > max_confidence_val {
                max_confidence_val = c_val;
                max_confidence = f.confidence.clone();
            }
        }

        // If multiple modules reported, we can bump confidence across the canonical group
        if modules_set.len() > 1
            && max_confidence_val < 2 {
                max_confidence = ConfidenceLevel::Firm; 
            }

        let contributing_modules: Vec<SourceModule> = modules_set.into_iter().collect();
        let affected_routes: Vec<String> = routes_set.into_iter().collect();

        CanonicalFinding {
            id: Uuid::new_v4().to_string(),
            canonical_slug: slug,
            title,
            risk_family,
            severity: max_severity,
            confidence: max_confidence,
            merged_evidence_count: total_evidence,
            contributing_modules,
            affected_routes,
            underlying_findings: group_findings.to_vec(),
            verification_status: VerificationStatus::Unverified,
            exploitability_score: None,
            exploitability_level: None,
            exploitability_reasoning: None,
            attack_surface_tags: Vec::new(),
            active_verification: None,
            confidence_calibration: None,
            priority_assessment: None,
        }
    }

    fn normalize_title(&self, raw: &str) -> String {
        let lower = raw.to_lowercase();
        if lower.contains("strict-transport-security") || lower.contains("hsts") {
            "Missing Strict-Transport-Security (HSTS)".to_string()
        } else if lower.contains("content-security-policy") || lower.contains("csp") {
            "Content-Security-Policy Misconfiguration".to_string()
        } else if lower.contains("x-frame-options") {
            "Missing X-Frame-Options Header".to_string()
        } else if lower.contains("cookie") {
            "Insecure Cookie Configuration".to_string()
        } else if lower.contains("cors") || lower.contains("access-control") {
            "Permissive Cross-Origin Resource Sharing (CORS)".to_string()
        } else {
            raw.to_string()
        }
    }

    fn severity_val(&self, s: &SeverityLevel) -> i32 {
        match s {
            SeverityLevel::Critical => 4,
            SeverityLevel::High => 3,
            SeverityLevel::Medium => 2,
            SeverityLevel::Low => 1,
            SeverityLevel::Informational => 0,
        }
    }

    fn confidence_val(&self, c: &ConfidenceLevel) -> i32 {
        match c {
            ConfidenceLevel::Certain => 3,
            ConfidenceLevel::Firm => 2,
            ConfidenceLevel::Tentative => 1,
            ConfidenceLevel::Low => 0,
        }
    }
}
