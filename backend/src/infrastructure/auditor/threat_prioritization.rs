use crate::domain::entities::*;

pub struct ThreatPrioritizationEngine;

impl ThreatPrioritizationEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn evaluate_all(&self, canonical_findings: &mut [CanonicalFinding], attack_paths: &mut [AttackPath], learning_engine: &crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine) {
        // Evaluate Attack Paths first
        for path in attack_paths.iter_mut() {
            let mut score: i32 = 40; // Base score for any correlated path
            let mut reasons = Vec::new();

            // Risk level from AttackPath (Actionable, Theoretical, Inert)
            match path.overall_risk_level {
                ExploitabilityLevel::Actionable => {
                    score += 30;
                    reasons.push("Actionable multi-step attack chain".to_string());
                },
                ExploitabilityLevel::Theoretical => {
                    score += 10;
                    reasons.push("Theoretical attack path".to_string());
                },
                ExploitabilityLevel::Inert => {
                    score -= 20;
                    reasons.push("Inert path - conditions cannot be met".to_string());
                }
            }

            // Path score boost
            if path.attack_path_score > 70 {
                score += 15;
                reasons.push("High correlation path score".to_string());
            }

            // Verification layer
            if let Some(av) = &path.active_verification {
                if av.status == VerificationStatus::VerifiedActionable {
                    score += 20;
                    reasons.push("Verified runtime exploitability across nodes".to_string());
                } else if av.status == VerificationStatus::VerifiedInert {
                    score -= 30;
                    reasons.push("Path verification failed - Inert".to_string());
                }
            } else {
                score -= 10;
                reasons.push("Path remains unverified by active probing".to_string());
            }

            // Context checking
            let ctx_joined = path.shared_context.join(" ").to_lowercase();
            if ctx_joined.contains("auth") || ctx_joined.contains("login") || ctx_joined.contains("session") || ctx_joined.contains("credential") {
                score += 20;
                reasons.push("Involves authentication or session boundaries".to_string());
            }

            let final_score = score.clamp(0, 100) as u8;
            path.priority_assessment = Some(PriorityAssessment {
                priority_score: final_score,
                priority_level: Self::score_to_level(final_score, &reasons),
                reasoning: reasons,
                learning_impact: None,
            });
        }

        // Sort Attack Paths
        attack_paths.sort_by(|a, b| {
            let s_a = a.priority_assessment.as_ref().map(|x| x.priority_score).unwrap_or(0);
            let s_b = b.priority_assessment.as_ref().map(|x| x.priority_score).unwrap_or(0);
            s_b.cmp(&s_a)
        });

        // Evaluate Canonical Findings
        for cf in canonical_findings.iter_mut() {
            let mut score: i32 = match cf.severity {
                SeverityLevel::Critical => 60,
                SeverityLevel::High => 45,
                SeverityLevel::Medium => 30,
                SeverityLevel::Low => 15,
                SeverityLevel::Informational => 5,
            };
            let mut reasons = Vec::new();

            reasons.push(format!("Base severity: {:?}", cf.severity));

            // Exploitability
            if let Some(lvl) = &cf.exploitability_level {
                if lvl == &ExploitabilityLevel::Actionable {
                    score += 20;
                    reasons.push("Actionable standalone exploitability".to_string());
                } else if lvl == &ExploitabilityLevel::Inert {
                    score -= 20;
                    reasons.push("Low isolated exploitability".to_string());
                }
            }

            // Confidence DB
            if let Some(cal) = &cf.confidence_calibration {
                if cal.reliability_coefficient < 0.6 {
                    score -= 15;
                    reasons.push("Penalized due to historically low pattern reliability".to_string());
                } else if cal.reliability_coefficient > 1.2 {
                    score += 10;
                    reasons.push("Boosted due to historically high pattern reliability".to_string());
                }
            }

            // Attack chain linkage check
            let is_in_path = attack_paths.iter().any(|ap| ap.involved_canonical_slugs.contains(&cf.canonical_slug));
            if is_in_path {
                score += 15;
                reasons.push("Critical component of a correlated Attack Path".to_string());
            }

            // Sensitive Surface Context
            let has_sensitive_surface = cf.attack_surface_tags.iter().any(|t| {
                let tag = t.to_lowercase();
                tag.contains("auth") || tag.contains("login") || tag.contains("admin") || tag.contains("api")
            });
            if has_sensitive_surface {
                score += 15;
                reasons.push("Affects high-value surface (Auth/Admin/API)".to_string());
            } else if cf.severity == SeverityLevel::High || cf.severity == SeverityLevel::Critical {
                // High severity but non-sensitive, so dampen
                score -= 10;
                reasons.push("Dampened: High severity but lacks sensitive surface exposure".to_string());
            }

            // Active Verification Overrides
            if let Some(av) = &cf.active_verification {
                if av.status == VerificationStatus::VerifiedActionable {
                    score += 20;
                    reasons.push("Positively verified via runtime probe".to_string());
                } else if av.status == VerificationStatus::VerifiedInert {
                    // Massive penalty
                    score -= 40;
                    reasons.push("Runtime verification disproved finding impact".to_string());
                } else if av.status == VerificationStatus::PartiallyVerified {
                    score -= 5; // Slight dip
                    reasons.push("Inconsistently verified across routes".to_string());
                }
            } else {
                score -= 10;
                reasons.push("Remains unverified by runtime engine".to_string());
            }

            // Apply Learning Modifier
            let sig = crate::infrastructure::auditor::confidence_calibration::ConfidenceCalibrationEngine::generate_signature(cf);
            let learning_impact = learning_engine.generate_impact(&sig).await;
            
            if learning_impact.priority_modifier != 0 {
                score += learning_impact.priority_modifier;
                if let Some(reason) = &learning_impact.reasoning {
                    reasons.push(format!("Learning Loop: {}", reason));
                }
            }

            let final_score = score.clamp(0, 100) as u8;
            cf.priority_assessment = Some(PriorityAssessment {
                priority_score: final_score,
                priority_level: Self::score_to_level(final_score, &reasons),
                reasoning: reasons,
                learning_impact: learning_impact.reasoning,
            });
        }

        // Sort Canonical Findings
        canonical_findings.sort_by(|a, b| {
            let s_a = a.priority_assessment.as_ref().map(|x| x.priority_score).unwrap_or(0);
            let s_b = b.priority_assessment.as_ref().map(|x| x.priority_score).unwrap_or(0);
            s_b.cmp(&s_a)
        });
    }

    fn score_to_level(score: u8, _reasons: &Vec<String>) -> PriorityLevel {
        if score >= 85 {
            PriorityLevel::Critical
        } else if score >= 60 {
            PriorityLevel::High
        } else if score >= 35 {
            PriorityLevel::Medium
        } else {
            PriorityLevel::Low
        }
    }
}
