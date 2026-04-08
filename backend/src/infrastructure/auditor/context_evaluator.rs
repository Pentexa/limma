use crate::domain::entities::*;

pub struct ContextEvaluator;

impl ContextEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate_all(&self, findings: &mut Vec<SecurityAuditFinding>) -> ContextStats {
        let mut elevated = 0;
        let mut downgraded = 0;
        let mut suppressed = 0;
        let mut unchanged = 0;

        // Pre-compute cluster summaries for dedup detection
        let summary_counts: std::collections::HashMap<String, usize> = {
            let mut map = std::collections::HashMap::new();
            for f in findings.iter() {
                let key = Self::dedup_key(f);
                *map.entry(key).or_insert(0) += 1;
            }
            map
        };

        for finding in findings.iter_mut() {
            let assessment = self.evaluate_single(finding, &summary_counts);
            match assessment.adjustment {
                PriorityAdjustment::Elevated => elevated += 1,
                PriorityAdjustment::Downgraded => downgraded += 1,
                PriorityAdjustment::Suppressed => suppressed += 1,
                PriorityAdjustment::Unchanged => unchanged += 1,
            }

            // Hard Severity Dampening
            if assessment.noise_indicators.contains(&NoiseIndicator::StaticAssetHeaderMissing) || assessment.suppression_reason.is_some() {
                if finding.severity == SeverityLevel::Critical || finding.severity == SeverityLevel::High || finding.severity == SeverityLevel::Medium {
                    finding.severity = SeverityLevel::Low;
                }
            }

            finding.context_summary = Some(assessment.context_summary.clone());
            finding.context_assessment = Some(assessment);
        }

        // Re-sort: suppressed go to bottom, rest by adjusted score desc
        findings.sort_by(|a, b| {
            let a_ctx = a.context_assessment.as_ref();
            let b_ctx = b.context_assessment.as_ref();
            let a_supp = a_ctx.map(|c| c.adjustment == PriorityAdjustment::Suppressed).unwrap_or(false);
            let b_supp = b_ctx.map(|c| c.adjustment == PriorityAdjustment::Suppressed).unwrap_or(false);
            if a_supp != b_supp {
                return a_supp.cmp(&b_supp); // non-suppressed first
            }
            let sa = a_ctx.map(|c| c.adjusted_score).unwrap_or(0);
            let sb = b_ctx.map(|c| c.adjusted_score).unwrap_or(0);
            sb.cmp(&sa)
        });

        ContextStats { elevated, downgraded, suppressed, unchanged }
    }

    fn dedup_key(f: &SecurityAuditFinding) -> String {
        format!("{:?}::{}", f.category, f.summary.to_lowercase().chars().take(40).collect::<String>())
    }

    fn evaluate_single(
        &self,
        finding: &mut SecurityAuditFinding,
        summary_counts: &std::collections::HashMap<String, usize>,
    ) -> ContextAwareAssessment {
        let mut signals: Vec<ContextSignal> = Vec::new();
        let mut noise: Vec<NoiseIndicator> = Vec::new();
        let mut delta: i32 = 0;

        let base_score = finding.risk_score.as_ref().map(|s| s.total_score).unwrap_or(30);
        let combined = format!("{} {}", finding.summary, finding.technical_details).to_lowercase();

        // ═══════════════════════════════════════
        // ESCALATION SIGNALS (increase priority)
        // ═══════════════════════════════════════

        // Auth surface detection
        if combined.contains("auth") || combined.contains("login") || combined.contains("session")
            || combined.contains("credential") || combined.contains("password") || combined.contains("oauth") {
            signals.push(ContextSignal::AuthenticationSurface);
            delta += 8;
        }

        // Cookie/Session evidence
        if combined.contains("cookie") || combined.contains("set-cookie") || combined.contains("session")
            || combined.contains("httponly") || combined.contains("samesite") {
            signals.push(ContextSignal::SessionOrCookieRelevance);
            delta += 6;
        }

        // Admin path exposure
        if combined.contains("/admin") || combined.contains("/dashboard") || combined.contains("/manage")
            || combined.contains("/config") || combined.contains("wp-admin") || combined.contains("/console") {
            signals.push(ContextSignal::AdministrativePathExposed);
            delta += 10;
        }

        // Dangerous method on sensitive path
        if let Some(ref method) = finding.method {
            let m = method.to_uppercase();
            if (m == "DELETE" || m == "PUT" || m == "PATCH") && finding.affected_path_or_endpoint.is_some() {
                signals.push(ContextSignal::DangerousMethodOnSensitivePath);
                delta += 8;
            }
        }

        // Multi-module confirmation
        if finding.correlation_count >= 2
            && finding.correlation_type == Some(CorrelationType::CompoundRisk) {
            signals.push(ContextSignal::MultiModuleConfirmation);
            delta += 10;
        }

        // Concrete evidence with validation
        let has_strong_evidence = finding.evidence.len() >= 2
            && finding.evidence.iter().any(|e| e.validation_context.is_some());
        if has_strong_evidence {
            signals.push(ContextSignal::ConcreteExploitEvidence);
            delta += 5;
        }

        // High confidence chain
        if finding.confidence == ConfidenceLevel::Certain && finding.correlation_count > 0 {
            signals.push(ContextSignal::HighConfidenceChain);
            delta += 5;
        }

        // Sensitive endpoint categories
        if finding.category == FindingCategory::AuthenticationBypass
            || finding.category == FindingCategory::SuspiciousEndpoint {
            signals.push(ContextSignal::SensitiveEndpointExposed);
            delta += 5;
        }

        // ═══════════════════════════════════════
        // NOISE INDICATORS (decrease priority)
        // ═══════════════════════════════════════

        // Generic infrastructure observation with no exploit path
        if finding.category == FindingCategory::InformationDisclosure
            && finding.confidence != ConfidenceLevel::Certain
            && finding.evidence.is_empty() {
            noise.push(NoiseIndicator::GenericInfraObservation);
            delta -= 12;
        }

        // Low confidence isolated finding (no correlation support)
        if (finding.confidence == ConfidenceLevel::Low || finding.confidence == ConfidenceLevel::Tentative)
            && finding.correlation_count == 0
            && finding.evidence.len() <= 1 {
            noise.push(NoiseIndicator::LowConfidenceIsolated);
            delta -= 10;
        }

        // Speculative impact with no evidence
        if finding.evidence.is_empty() && combined.contains("[real world impact]") {
            noise.push(NoiseIndicator::SpeculativeImpactNoEvidence);
            delta -= 8;
        }

        // Duplicate header finding in same cluster
        let dup_key = Self::dedup_key(finding);
        if let Some(&count) = summary_counts.get(&dup_key) {
            if count > 1 && finding.category == FindingCategory::SecurityMisconfiguration {
                noise.push(NoiseIndicator::DuplicateHeaderInCluster);
                delta -= 5;
            }
        }

        // Weak correlation with low overlap
        if finding.correlation_count == 1
            && finding.correlation_confidence == Some(ConfidenceLevel::Low) {
            noise.push(NoiseIndicator::WeakCorrelationOverlap);
            delta -= 6;
        }

        // Overly broad category with no specifics
        if finding.affected_path_or_endpoint.is_none()
            && finding.method.is_none()
            && finding.protocol.is_none()
            && finding.category == FindingCategory::InformationDisclosure
            && finding.confidence != ConfidenceLevel::Certain {
            noise.push(NoiseIndicator::OverlyBroadCategory);
            delta -= 8;
        }

        // Static Asset / Weak Context Dampening
        let mut is_sensitive_route = false;
        if let Some(ref path) = finding.affected_path_or_endpoint {
            let p = path.to_lowercase();
            if p.contains("login") || p.contains("admin") || p.contains("account") || p.contains("payment") || p.contains("api") || p.contains("auth") {
                is_sensitive_route = true;
            }
        } else if combined.contains("login") || combined.contains("admin") || combined.contains("account") || combined.contains("payment") || combined.contains("api") || combined.contains("auth") {
            is_sensitive_route = true;
        }

        let has_dynamic_content = combined.contains("input") || combined.contains("reflected") || combined.contains("script");

        if !is_sensitive_route && !has_dynamic_content && !signals.contains(&ContextSignal::AuthenticationSurface) {
            noise.push(NoiseIndicator::StaticAssetHeaderMissing);
            delta -= 15;
        }

        // ═══════════════════════════════════════
        // SUPPRESSION CHECK
        // ═══════════════════════════════════════

        let mut suppression_reason: Option<SuppressionReason> = None;

        // Suppress if noise overwhelms signals and adjusted score is negligible
        let adjusted_raw = (base_score as i32 + delta).clamp(0, 100) as u32;

        if adjusted_raw < 10 && noise.len() >= 2 && signals.is_empty() {
            suppression_reason = Some(SuppressionReason::ZeroExploitRelevance);
        }

        if noise.contains(&NoiseIndicator::GenericInfraObservation)
            && noise.contains(&NoiseIndicator::LowConfidenceIsolated)
            && signals.is_empty() {
            suppression_reason = Some(SuppressionReason::GenericNonActionable);
        }

        if noise.contains(&NoiseIndicator::SpeculativeImpactNoEvidence)
            && finding.evidence.is_empty()
            && finding.confidence == ConfidenceLevel::Low {
            suppression_reason = Some(SuppressionReason::InsufficientEvidence);
        }

        // ═══════════════════════════════════════
        // FINAL ASSESSMENT
        // ═══════════════════════════════════════

        let adjustment = if suppression_reason.is_some() {
            PriorityAdjustment::Suppressed
        } else if delta > 5 {
            PriorityAdjustment::Elevated
        } else if delta < -5 {
            PriorityAdjustment::Downgraded
        } else {
            PriorityAdjustment::Unchanged
        };

        let adjusted_score = if suppression_reason.is_some() { 0 } else { adjusted_raw };
        let adjusted_level = Self::score_to_level(adjusted_score);

        // Dynamically compute evidence weight
        let ev_count = finding.evidence.len();
        let has_validation = finding.evidence.iter().any(|e| e.validation_context.is_some());
        
        let evidence_weight = if has_validation && ev_count >= 1 || finding.confidence == ConfidenceLevel::Certain {
            EvidenceWeight::Strong
        } else if ev_count > 0 {
            EvidenceWeight::Moderate
        } else if finding.confidence != ConfidenceLevel::Certain {
            EvidenceWeight::Weak
        } else {
            EvidenceWeight::Zero
        };
        finding.evidence_weight = Some(evidence_weight);

        // Dynamically compute exploitability
        let exploitability = if signals.contains(&ContextSignal::ConcreteExploitEvidence) {
            Exploitability::Proven
        } else if signals.contains(&ContextSignal::HighConfidenceChain) || signals.contains(&ContextSignal::AuthenticationSurface) || signals.contains(&ContextSignal::AdministrativePathExposed) {
            Exploitability::Likely
        } else if suppression_reason.is_some() || noise.contains(&NoiseIndicator::StaticAssetHeaderMissing) {
            Exploitability::Unlikely
        } else if delta < 0 {
            Exploitability::Possible
        } else {
            Exploitability::Possible
        };
        finding.exploitability = Some(exploitability);

        let context_summary = self.generate_summary(&adjustment, &signals, &noise, &suppression_reason);

        ContextAwareAssessment {
            adjustment,
            score_delta: delta,
            adjusted_score,
            adjusted_level,
            signals,
            noise_indicators: noise,
            suppression_reason,
            context_summary,
        }
    }

    fn score_to_level(score: u32) -> RiskLevel {
        match score {
            90..=100 => RiskLevel::Critical,
            70..=89 => RiskLevel::High,
            45..=69 => RiskLevel::Medium,
            20..=44 => RiskLevel::Low,
            _ => RiskLevel::Info,
        }
    }

    fn generate_summary(
        &self,
        adj: &PriorityAdjustment,
        signals: &[ContextSignal],
        noise: &[NoiseIndicator],
        suppression: &Option<SuppressionReason>,
    ) -> String {
        match adj {
            PriorityAdjustment::Suppressed => {
                let reason = match suppression {
                    Some(SuppressionReason::DuplicateClusterNoise) => "duplicate cluster noise",
                    Some(SuppressionReason::ZeroExploitRelevance) => "zero real-world exploit relevance",
                    Some(SuppressionReason::InsufficientEvidence) => "insufficient evidence to justify priority",
                    Some(SuppressionReason::GenericNonActionable) => "generic non-actionable observation",
                    None => "multiple noise indicators with no supporting context",
                };
                format!("Suppressed: {}", reason)
            }
            PriorityAdjustment::Elevated => {
                let reasons: Vec<&str> = signals.iter().map(|s| match s {
                    ContextSignal::AuthenticationSurface => "authentication surface is affected",
                    ContextSignal::SessionOrCookieRelevance => "cookie/session context is relevant",
                    ContextSignal::SensitiveEndpointExposed => "sensitive endpoint is exposed",
                    ContextSignal::DangerousMethodOnSensitivePath => "dangerous HTTP method on sensitive path",
                    ContextSignal::MultiModuleConfirmation => "confirmed by multiple analysis modules",
                    ContextSignal::ConcreteExploitEvidence => "backed by concrete exploit evidence",
                    ContextSignal::AdministrativePathExposed => "administrative path is exposed",
                    ContextSignal::HighConfidenceChain => "high-confidence correlation chain",
                }).collect();
                format!("Elevated: {}", reasons.join(", "))
            }
            PriorityAdjustment::Downgraded => {
                let reasons: Vec<&str> = noise.iter().map(|n| match n {
                    NoiseIndicator::GenericInfraObservation => "generic infrastructure observation",
                    NoiseIndicator::WeakCorrelationOverlap => "weak correlation overlap",
                    NoiseIndicator::DuplicateHeaderInCluster => "duplicate header in same cluster",
                    NoiseIndicator::SpeculativeImpactNoEvidence => "speculative impact without evidence",
                    NoiseIndicator::LowConfidenceIsolated => "low confidence isolated finding",
                    NoiseIndicator::OverlyBroadCategory => "overly broad category with no specifics",
                    NoiseIndicator::StaticAssetHeaderMissing => "missing header on static/low-risk asset",
                    NoiseIndicator::RedundantFinding => "redundant with higher-priority finding",
                }).collect();
                format!("Downgraded: {}", reasons.join(", "))
            }
            PriorityAdjustment::Unchanged => {
                "Context analysis found no significant escalation or noise factors.".to_string()
            }
        }
    }
}
