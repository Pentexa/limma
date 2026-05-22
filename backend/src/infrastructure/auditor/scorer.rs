use crate::domain::entities::*;

pub struct RiskScorer;

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score_finding(&self, finding: &SecurityAuditFinding) -> RiskScore {
        let mut contributions: Vec<RiskContribution> = Vec::new();
        let mut correlation_score: u32 = 0;
        let mut raw: i32 = 0;

        // ── 1. Base Severity ──
        let base = match finding.severity {
            SeverityLevel::Critical => 40,
            SeverityLevel::High => 30,
            SeverityLevel::Medium => 20,
            SeverityLevel::Low => 10,
            SeverityLevel::Informational => 5,
        };
        contributions.push(RiskContribution {
            factor: RiskFactor::BaseSeverity,
            delta: base,
            explanation: format!("Base severity: {:?}", finding.severity),
        });
        raw += base;

        // ── 2. Confidence Multiplier ──
        let conf_multiplier = match finding.confidence {
            ConfidenceLevel::Certain => 1.0,
            ConfidenceLevel::High => 0.95,
            ConfidenceLevel::Firm => 0.9,
            ConfidenceLevel::Medium => 0.85,
            ConfidenceLevel::Tentative => 0.8,
            ConfidenceLevel::Low => 0.7,
        };

        let scaled_base = (base as f32 * conf_multiplier) as i32;
        let conf_delta = scaled_base - base;

        if conf_delta != 0 {
            contributions.push(RiskContribution {
                factor: if conf_delta > 0 {
                    RiskFactor::ConfidenceMultiplier
                } else {
                    RiskFactor::LowConfidencePenalty
                },
                delta: conf_delta,
                explanation: format!(
                    "Confidence level multiplier ({:.2}x): {:?}",
                    conf_multiplier, finding.confidence
                ),
            });
            raw += conf_delta;
        }

        // ── 3. Evidence Quality & Weight ──
        let ev_count = finding.evidence.len();
        let has_validation = finding
            .evidence
            .iter()
            .any(|e| e.validation_context.is_some());

        if let Some(ref weight) = finding.evidence_weight {
            match weight {
                EvidenceWeight::Strong => {
                    contributions.push(RiskContribution {
                        factor: RiskFactor::EvidenceQuality,
                        delta: 15,
                        explanation:
                            "Strong, high-fidelity evidence directly supports this finding"
                                .to_string(),
                    });
                    raw += 15;
                }
                EvidenceWeight::Weak | EvidenceWeight::Zero => {
                    contributions.push(RiskContribution {
                        factor: RiskFactor::WeakEvidencePenalty,
                        delta: -15,
                        explanation: "Weak or generic contextual evidence".to_string(),
                    });
                    raw -= 15;
                }
                _ => {}
            }
        } else {
            if ev_count >= 2 && has_validation {
                contributions.push(RiskContribution {
                    factor: RiskFactor::EvidenceQuality,
                    delta: 15,
                    explanation: format!(
                        "{} evidence items with active validation context",
                        ev_count
                    ),
                });
                raw += 15;
            } else if ev_count >= 1 {
                contributions.push(RiskContribution {
                    factor: RiskFactor::EvidenceQuality,
                    delta: 5,
                    explanation: format!("{} evidence item(s) attached", ev_count),
                });
                raw += 5;
            } else {
                contributions.push(RiskContribution {
                    factor: RiskFactor::WeakEvidencePenalty,
                    delta: -10,
                    explanation: "No concrete evidence provided".to_string(),
                });
                raw -= 10;
            }
        }

        // Intermediate clamp checkpoint (raw is further adjusted below)

        let summary_lower = finding.summary.to_lowercase();
        let details_lower = finding.technical_details.to_lowercase();
        let combined = format!("{} {}", summary_lower, details_lower);

        // ── 4. Correlation → Confidence (NOT Severity) ──
        // Correlation confirms the issue is real and systemic, but does NOT inflate severity.
        // Severity must come from exploit signals (auth, dangerous methods, sensitive paths, etc.)
        if finding.correlation_count > 0 {
            let corr_confidence = match finding.correlation_count {
                1 => 10,
                2 => 18,
                3..=5 => 25,
                _ => 30,
            };
            let mut corr_total = corr_confidence;

            if finding.correlation_type == Some(CorrelationType::CompoundRisk)
                && !finding.correlation_is_hygiene_gap
            {
                corr_total += 10;
            }

            correlation_score = corr_total.clamp(0, 50) as u32;

            // Correlation is recorded for transparency, but with delta=0 (no severity impact)
            contributions.push(RiskContribution {
                factor: RiskFactor::CorrelationBoost,
                delta: 0,
                explanation: format!(
                    "Correlated with {} sibling signals (increases confidence, not severity). Confidence strength: {}",
                    finding.correlation_count, correlation_score
                ),
            });

            // Systemic finding rule: correlated but NO exploit indicators => cap at Low severity
            let has_exploit_signals = self.has_exploit_indicators(finding, &combined);
            if !has_exploit_signals && raw > 30 {
                let cap_delta = raw - 30;
                contributions.push(RiskContribution {
                        factor: RiskFactor::NoisyCorrelationPenalty,
                        delta: -cap_delta,
                        explanation: "Systemic finding without exploit indicators: capped at Low severity, High confidence".to_string(),
                    });
                raw -= cap_delta;
            }
        }

        // ── 5. Sensitive Endpoint / Auth Boost ──

        if combined.contains("auth")
            || combined.contains("login")
            || combined.contains("session")
            || combined.contains("token")
            || combined.contains("credential")
            || combined.contains("password")
        {
            contributions.push(RiskContribution {
                factor: RiskFactor::AuthenticationExposure,
                delta: 15,
                explanation: "Authentication or session-related exposure detected".to_string(),
            });
            raw += 15;
        }

        if finding.category == FindingCategory::SuspiciousEndpoint
            || finding.category == FindingCategory::AuthenticationBypass
        {
            contributions.push(RiskContribution {
                factor: RiskFactor::SensitiveEndpoint,
                delta: 10,
                explanation: format!("Sensitive endpoint category: {:?}", finding.category),
            });
            raw += 10;
        }

        // ── 6. Dangerous Method Boost ──
        if let Some(ref method) = finding.method {
            let m = method.to_uppercase();
            if m == "DELETE" || m == "PUT" || m == "PATCH" {
                contributions.push(RiskContribution {
                    factor: RiskFactor::DangerousMethod,
                    delta: 10,
                    explanation: format!("Dangerous HTTP method: {}", m),
                });
                raw += 10;
            }
        }

        // ── 7. Generic Match Penalty ──
        if finding.confidence == ConfidenceLevel::Tentative && finding.evidence.is_empty() {
            contributions.push(RiskContribution {
                factor: RiskFactor::GenericMatchPenalty,
                delta: -8,
                explanation: "Low-fidelity match with no supporting evidence".to_string(),
            });
            raw -= 8;
        }

        // ── 8. Header Risk Dampening ──
        if finding.category == FindingCategory::SecurityMisconfiguration
            && finding.evidence_weight != Some(EvidenceWeight::Strong)
        {
            // dampen score to max out at Low priority (e.g. max 44).
            if raw >= 45 {
                let damp_amount = raw - 40;
                contributions.push(RiskContribution {
                        factor: RiskFactor::GenericMatchPenalty,
                        delta: -damp_amount,
                        explanation: "Defaulting missing header impact to Low severity due to lacking exploit context or strong evidence".to_string(),
                    });
                raw -= damp_amount;
            }
        }

        let finding_score = raw.clamp(0, 100) as u32;

        // ── Clamp & Map to Level ──
        let total_score = raw.clamp(0, 100) as u32;
        let mut level = Self::score_to_level(total_score);
        let mut escalation_reason = None;

        let base_level_score = match finding.severity {
            SeverityLevel::Critical => 90,
            SeverityLevel::High => 70,
            SeverityLevel::Medium => 45,
            SeverityLevel::Low => 20,
            SeverityLevel::Informational => 0,
        };
        let base_level = Self::score_to_level(base_level_score);

        // Escalation checks — correlation alone is NEVER sufficient
        if Self::level_val(&level) > Self::level_val(&base_level) {
            let has_exploit_signals = self.has_exploit_indicators(finding, &combined);
            let has_strong_evidence = finding.evidence_weight == Some(EvidenceWeight::Strong);

            if has_exploit_signals && has_strong_evidence {
                escalation_reason = Some(format!(
                    "Severity escalated from {:?} to {:?}: exploit signals and strong evidence confirmed.",
                    base_level, level
                ));
            } else if has_exploit_signals {
                escalation_reason = Some(format!(
                    "Severity escalated from {:?} to {:?}: exploit indicators present (auth surface, dangerous method, or sensitive endpoint).",
                    base_level, level
                ));
            } else {
                level = base_level.clone();
                escalation_reason = Some(format!(
                    "Prevented escalation from {:?}. Correlation alone cannot increase severity without exploit signals.",
                    base_level
                ));
            }
        }

        // ── Aggressive Risk Dampening ──
        let mut final_total = total_score;
        let mut final_finding_score = finding_score;
        let has_exploit_signals = self.has_exploit_indicators(finding, &combined);
        let has_dynamic_interaction = combined.contains("input")
            || combined.contains("script")
            || combined.contains("parameter")
            || combined.contains("execute")
            || combined.contains("reflected");
        let has_weak_evidence = finding.evidence_weight != Some(EvidenceWeight::Strong);

        let is_weak_context = !has_exploit_signals && !has_dynamic_interaction;
        let is_hygiene_issue = is_weak_context || (has_weak_evidence && !has_exploit_signals);

        if is_hygiene_issue {
            if Self::level_val(&level) >= Self::level_val(&RiskLevel::Medium) {
                // Must downgrade to Low (max 44)
                let drop_amount = final_total.saturating_sub(44);
                if drop_amount > 0 {
                    final_total = 44;
                    final_finding_score = final_finding_score.min(44);

                    contributions.push(RiskContribution {
                        factor: RiskFactor::GenericMatchPenalty,
                        delta: -(drop_amount as i32),
                        explanation: "Aggressive Dampening: finding lacks dynamic input, execution path, or sensitive data context.".to_string(),
                    });
                }
                level = RiskLevel::Low;
                escalation_reason = Some("Downgraded to Hygiene Issue: Weak evidence combined with weak context prevents Medium or higher severity.".to_string());
            }

            // If it's truly empty of evidence, push it all the way to Informational (max 19)
            if finding.evidence.is_empty() && final_total > 19 {
                let drop = final_total - 19;
                final_total = 19;
                final_finding_score = final_finding_score.min(19);
                contributions.push(RiskContribution {
                    factor: RiskFactor::WeakEvidencePenalty,
                    delta: -(drop as i32),
                    explanation:
                        "Informational Dampening: Hygiene issue with zero evidentiary support."
                            .to_string(),
                });
                level = RiskLevel::Info;
            }
        }

        let mut priority_statement =
            self.generate_priority_statement(finding, final_total, &contributions);
        if is_hygiene_issue && finding.category == FindingCategory::SecurityMisconfiguration {
            priority_statement = format!("Hygiene Issue: {}", priority_statement);
        } else if is_hygiene_issue {
            priority_statement = format!("Low-Exploitability Weakness: {}", priority_statement);
        }

        RiskScore {
            finding_score: final_finding_score,
            correlation_score,
            total_score: final_total,
            level,
            contributions,
            priority_statement,
            escalation_reason,
        }
    }

    fn has_exploit_indicators(&self, finding: &SecurityAuditFinding, combined_text: &str) -> bool {
        let has_auth = combined_text.contains("auth")
            || combined_text.contains("login")
            || combined_text.contains("session")
            || combined_text.contains("credential")
            || combined_text.contains("password");

        let is_sensitive = finding.category == FindingCategory::SuspiciousEndpoint
            || finding.category == FindingCategory::AuthenticationBypass;

        let has_dangerous_method = if let Some(ref m) = finding.method {
            let u = m.to_uppercase();
            u == "DELETE" || u == "PUT" || u == "PATCH"
        } else {
            false
        };

        let has_concrete_evidence = finding
            .evidence
            .iter()
            .any(|e| e.validation_context.is_some());

        has_auth || is_sensitive || has_dangerous_method || has_concrete_evidence
    }

    fn level_val(level: &RiskLevel) -> u32 {
        match level {
            RiskLevel::Critical => 4,
            RiskLevel::High => 3,
            RiskLevel::Medium => 2,
            RiskLevel::Low => 1,
            RiskLevel::Info => 0,
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

    fn generate_priority_statement(
        &self,
        finding: &SecurityAuditFinding,
        score: u32,
        contributions: &[RiskContribution],
    ) -> String {
        let has_correlation = contributions
            .iter()
            .any(|c| c.factor == RiskFactor::CorrelationBoost);
        let has_auth = contributions
            .iter()
            .any(|c| c.factor == RiskFactor::AuthenticationExposure);
        let has_compound = contributions
            .iter()
            .any(|c| c.factor == RiskFactor::MultiModuleConfirmation);

        if score >= 90 {
            if has_compound {
                return "Critical priority: compound risk confirmed by multiple independent analysis modules.".to_string();
            }
            if has_auth {
                return "Critical priority: authentication-related exposure with high confidence evidence.".to_string();
            }
            return "Critical priority: severe vulnerability with strong evidence base."
                .to_string();
        }

        if score >= 70 {
            if has_correlation {
                return format!("High priority: multi-signal confirmation ({} correlated signals strengthen this finding).", finding.correlation_count);
            }
            if has_auth {
                return "High priority: sensitive authentication surface is exposed.".to_string();
            }
            return "High priority: significant security weakness detected with reliable evidence."
                .to_string();
        }

        if score >= 45 {
            if has_correlation {
                return "Medium priority: elevated risk due to supporting correlation signals."
                    .to_string();
            }
            return "Medium priority: notable security observation that should be reviewed."
                .to_string();
        }

        if score >= 20 {
            return "Low priority: minor observation with limited immediate impact.".to_string();
        }

        "Informational: general hardening recommendation with no confirmed exploit path."
            .to_string()
    }

    pub fn score_all(&self, findings: &mut [SecurityAuditFinding]) -> ScoringStats {
        let mut boosted = 0;
        let mut downgraded = 0;

        for finding in findings.iter_mut() {
            let score = self.score_finding(finding);

            let has_boost = score.contributions.iter().any(|c| {
                c.delta > 0
                    && matches!(
                        c.factor,
                        RiskFactor::CorrelationBoost
                            | RiskFactor::MultiModuleConfirmation
                            | RiskFactor::AuthenticationExposure
                            | RiskFactor::SensitiveEndpoint
                            | RiskFactor::DangerousMethod
                    )
            });
            let has_penalty = score.contributions.iter().any(|c| c.delta < 0);

            if has_boost {
                boosted += 1;
            }
            if has_penalty {
                downgraded += 1;
            }

            finding.risk_score = Some(score);
        }

        // Sort by score descending
        findings.sort_by(|a, b| {
            let sa = a.risk_score.as_ref().map(|s| s.total_score).unwrap_or(0);
            let sb = b.risk_score.as_ref().map(|s| s.total_score).unwrap_or(0);
            sb.cmp(&sa)
        });

        let total_scored = findings.len();

        let mut overall_risk_score: f64 = 0.0;
        if total_scored > 0 {
            // Base the domain's risk primarily on its most severe vulnerability
            overall_risk_score = findings[0]
                .risk_score
                .as_ref()
                .map(|s| s.total_score as f64)
                .unwrap_or(0.0);

            let mut hygiene_issues_seen = 0.0;

            // Add diminishing margins for the rest, completely capping hygiene gap contributions
            for (i, finding) in findings.iter().skip(1).enumerate() {
                let s = finding
                    .risk_score
                    .as_ref()
                    .map(|sc| sc.total_score)
                    .unwrap_or(0) as f64;
                let is_hygiene = finding
                    .risk_score
                    .as_ref()
                    .map(|sc| {
                        sc.priority_statement.contains("Hygiene Issue")
                            || sc
                                .priority_statement
                                .contains("Low-Exploitability Weakness")
                    })
                    .unwrap_or(false);

                let weight = if is_hygiene {
                    hygiene_issues_seen += 1.0;
                    if hygiene_issues_seen > 5.0 {
                        0.0 // Hard cap on minor hygiene issues contributing to overall domain risk
                    } else {
                        f64::min(0.5 / hygiene_issues_seen, s * 0.01) // Marginally increment
                    }
                } else if s >= 70.0 {
                    f64::min(s * 0.1, 10.0) / ((i as f64) + 1.0)
                } else if s >= 45.0 {
                    f64::min(s * 0.05, 5.0) / ((i as f64) + 1.0)
                } else {
                    f64::min(s * 0.02, 2.0) / ((i as f64) + 1.0)
                };

                overall_risk_score += weight;
            }

            overall_risk_score = overall_risk_score.min(100.0);
        }

        let top_risk_summary = findings.first().map(|f| {
            format!(
                "{} (Score: {})",
                f.summary,
                f.risk_score.as_ref().map(|s| s.total_score).unwrap_or(0)
            )
        });

        ScoringStats {
            total_scored,
            boosted,
            downgraded,
            overall_risk_score,
            top_risk_summary,
        }
    }
}
