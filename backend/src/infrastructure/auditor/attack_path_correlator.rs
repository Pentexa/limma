use crate::domain::entities::*;
use std::collections::HashSet;
use uuid::Uuid;

pub struct AttackPathEngine;

impl AttackPathEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn build_paths(&self, findings: &[CanonicalFinding]) -> Vec<AttackPath> {
        let mut paths = Vec::new();

        // Cross-examine findings to detect tactical exploit chains
        for f1 in findings {
            for f2 in findings {
                if f1.id == f2.id {
                    continue;
                }

                // E.g., Script Injection + Missing CSP
                if self.is_xss_escalation(f1, f2) {
                    if let Some(path) = self.build_xss_chain(f1, f2) {
                        paths.push(path);
                    }
                }

                // Missing Secure Cookie + General intercept risk (Weak TLS / no HSTS)
                if self.is_session_hijack(f1, f2) {
                    if let Some(path) = self.build_session_chain(f1, f2) {
                        paths.push(path);
                    }
                }
            }
        }

        // Deduplicate paths (quick hack, ideally standard graph traverse)
        let mut unique_paths = Vec::new();
        let mut seen = HashSet::new();
        for p in paths {
            let mut sorted_slugs = p.involved_canonical_slugs.clone();
            sorted_slugs.sort();
            let sig = format!("{}-{}", p.narrative, sorted_slugs.join("|"));
            if !seen.contains(&sig) {
                seen.insert(sig);
                unique_paths.push(p);
            }
        }

        // Sort by risk
        unique_paths.sort_by(|a, b| b.attack_path_score.cmp(&a.attack_path_score));
        unique_paths
    }

    fn is_xss_escalation(&self, inject: &CanonicalFinding, infra: &CanonicalFinding) -> bool {
        let is_reflect = inject.title.to_lowercase().contains("cross-site")
            || inject.title.to_lowercase().contains("xss")
            || inject.canonical_slug.contains("xss");

        let missing_csp = infra.canonical_slug.contains("missing-csp");

        is_reflect && missing_csp && self.shares_route_or_global(inject, infra)
    }

    fn build_xss_chain(
        &self,
        inject: &CanonicalFinding,
        infra: &CanonicalFinding,
    ) -> Option<AttackPath> {
        let shared: Vec<String> = self.get_shared_routes(inject, infra).into_iter().collect();
        let route_msg = if shared.is_empty() {
            "Global Scope".to_string()
        } else {
            format!("Route: {}", shared.join(", "))
        };

        Some(AttackPath {
            id: Uuid::new_v4().to_string(),
            attack_path_score: 80, // High severity path
            narrative: "Attacker can inject malicious scripts -> Lack of Content-Security-Policy (CSP) allows external script execution and data exfiltration -> Complete frontend takeover".to_string(),
            involved_canonical_slugs: vec![inject.canonical_slug.clone(), infra.canonical_slug.clone()],
            shared_context: vec![route_msg, "Frontend Security Boundary".to_string()],
            overall_risk_level: ExploitabilityLevel::Actionable,
            required_conditions: vec!["User interaction on targeted route".to_string()],
            active_verification: None,
            priority_assessment: None,
        })
    }

    fn is_session_hijack(&self, cookie: &CanonicalFinding, intercept: &CanonicalFinding) -> bool {
        let insecure_cookie = cookie.canonical_slug.contains("insecure-cookie");
        let network_interception = intercept.canonical_slug.contains("missing-hsts")
            || intercept.canonical_slug.contains("weak-tls");

        insecure_cookie && network_interception
    }

    fn build_session_chain(
        &self,
        cookie: &CanonicalFinding,
        intercept: &CanonicalFinding,
    ) -> Option<AttackPath> {
        let _shared: Vec<String> = self
            .get_shared_routes(cookie, intercept)
            .into_iter()
            .collect();
        let auth_related = cookie
            .attack_surface_tags
            .contains(&"requires_authentication".to_string())
            || intercept
                .attack_surface_tags
                .contains(&"requires_authentication".to_string());

        let mut score = 50;
        let mut level = ExploitabilityLevel::Theoretical;
        let mut reqs = vec!["Attacker resides on the same network or performs MitM".to_string()];

        if auth_related {
            score = 95;
            level = ExploitabilityLevel::Actionable;
            reqs.push("Active session occurs without forced encryption".to_string());
        }

        Some(AttackPath {
            id: Uuid::new_v4().to_string(),
            attack_path_score: score,
            narrative: "Attacker intercepts network traffic -> Missing HSTS/Weak TLS allows downgrade -> Gains access to session tokens due to missing Secure flag -> Complete Account Takeover".to_string(),
            involved_canonical_slugs: vec![cookie.canonical_slug.clone(), intercept.canonical_slug.clone()],
            shared_context: vec!["Network Protocol Layer".to_string(), "Session/Cookie Scope".to_string()],
            overall_risk_level: level,
            required_conditions: reqs,
            active_verification: None,
            priority_assessment: None,
        })
    }

    fn shares_route_or_global(&self, f1: &CanonicalFinding, f2: &CanonicalFinding) -> bool {
        if f1.affected_routes.is_empty() || f2.affected_routes.is_empty() {
            return true;
        }
        for r1 in &f1.affected_routes {
            if f2.affected_routes.contains(r1) {
                return true;
            }
        }
        false
    }

    fn get_shared_routes(&self, f1: &CanonicalFinding, f2: &CanonicalFinding) -> HashSet<String> {
        let s2: HashSet<_> = f2.affected_routes.iter().cloned().collect();
        let mut overlap = HashSet::new();
        for r in &f1.affected_routes {
            if s2.contains(r) {
                overlap.insert(r.clone());
            }
        }
        overlap
    }
}
