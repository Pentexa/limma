use crate::domain::entities::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CalibrationDB {
    pub patterns: HashMap<String, PatternCalibrationMetrics>,
}

pub struct ConfidenceCalibrationEngine {
    db_path: String,
    db: CalibrationDB,
}

impl ConfidenceCalibrationEngine {
    pub fn new() -> Self {
        let db_path = "calibration_db.json".to_string();
        let db = Self::load_db(&db_path);
        Self { db_path, db }
    }

    fn load_db(path: &str) -> CalibrationDB {
        if Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(db) = serde_json::from_str(&content) {
                    return db;
                }
            }
        }
        CalibrationDB::default()
    }

    fn save_db(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.db) {
            let _ = fs::write(&self.db_path, content);
        }
    }

    pub fn get_metrics(&self, signature: &str) -> Option<PatternCalibrationMetrics> {
        self.db.patterns.get(signature).cloned()
    }

    pub fn generate_signature(finding: &CanonicalFinding) -> String {
        let mut modules: Vec<String> = finding.contributing_modules.iter().map(|s| format!("{:?}", s)).collect();
        modules.sort();
        format!("{}_[{}]", finding.canonical_slug, modules.join(","))
    }

    pub fn update_from_scan(&mut self, canonical_findings: &mut Vec<CanonicalFinding>, learning_engine: &crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine) {
        for finding in canonical_findings.iter_mut() {
            let sig = Self::generate_signature(finding);
            let mut metrics = self.db.patterns.remove(&sig).unwrap_or_default();
            
            // Only update history if active verification resulted in some check
            if let Some(av) = &finding.active_verification {
                metrics.total_observations += 1;

                if av.status == VerificationStatus::VerifiedActionable {
                    metrics.successful_verifications += 1;
                } else if av.status == VerificationStatus::VerifiedInert {
                    metrics.failed_verifications += 1;
                } else if av.status == VerificationStatus::PartiallyVerified {
                    metrics.partial_verifications += 1;
                }

                // Recalculate average reproducibility (moving average)
                let current_rep = av.reproducibility_score as f32;
                if metrics.total_observations == 1 {
                    metrics.average_reproducibility = current_rep;
                } else {
                    // Weighted average prioritizing general history but acknowledging recent finding
                    metrics.average_reproducibility = ((metrics.average_reproducibility * (metrics.total_observations - 1) as f32) + current_rep) / metrics.total_observations as f32;
                }
            }

            self.db.patterns.insert(sig.clone(), metrics.clone());

            // Assign the ConfidenceCalibrationResult to the Canonical finding for UI visibility
            let mut reliability_coefficient = if metrics.total_observations > 0 {
                // If it successfully verified > 80% of the time, coefficient is high.
                // We use average_reproducibility as the base, scaled to 0.1 - 1.5
                let rep_ratio = metrics.average_reproducibility / 100.0;
                // e.g. 100% rep -> 1.5 multiplier, 50% rep -> 0.8 multiplier, 0% rep -> 0.1 multiplier
                0.1 + (rep_ratio * 1.4)
            } else {
                1.0 // Neutral
            };

            let original = finding.confidence.clone();
            let mut adjusted = original.clone();
            
            let mut impact = "Neutral - Insufficient History".to_string();
            let mut reasoning = "No historical calibration data exists for this pattern.".to_string();

            if metrics.total_observations >= 2 {
                if reliability_coefficient > 1.2 {
                    impact = "Confidence Boosted - Historically Reliable".to_string();
                    reasoning = format!("Pattern {} has a high empirical reproducibility score of {:.0}%. Confidence elevated.", sig, metrics.average_reproducibility);
                    // Upgrade confidence
                    adjusted = match original {
                        ConfidenceLevel::Tentative => ConfidenceLevel::Firm,
                        ConfidenceLevel::Firm => ConfidenceLevel::Certain,
                        _ => original.clone(),
                    }
                } else if reliability_coefficient < 0.6 {
                    impact = "Confidence Reduced - Historically Inconsistent".to_string();
                    reasoning = format!("Pattern {} frequently fails verification (Historical Rep.: {:.0}%). Confidence downgraded.", sig, metrics.average_reproducibility);
                    // Downgrade confidence
                    adjusted = match original {
                        ConfidenceLevel::Certain => ConfidenceLevel::Firm,
                        ConfidenceLevel::Firm => ConfidenceLevel::Tentative,
                        ConfidenceLevel::Tentative => ConfidenceLevel::Low,
                        _ => original.clone(),
                    }
                } else {
                    impact = "Confidence Maintained - Moderate History".to_string();
                    reasoning = format!("Pattern {} shows average reproducibility ({:.0}%). Maintaining baseline confidence.", sig, metrics.average_reproducibility);
                }
            }

            // Apply Learning Impact Overrides
            let learning_impact = learning_engine.generate_impact(&sig);
            reliability_coefficient = reliability_coefficient * learning_impact.confidence_multiplier;
            
            if learning_impact.confidence_multiplier < 0.9 || learning_impact.confidence_multiplier > 1.1 {
                // If the learning loop actively modified it, update the impact string.
               impact = format!("Confidence Modified by User Feedback").to_string();
            }

            finding.confidence = adjusted.clone();
            finding.confidence_calibration = Some(ConfidenceCalibrationResult {
                original_confidence: original,
                adjusted_confidence: adjusted,
                reliability_coefficient,
                calibration_impact: impact,
                reasoning,
                learning_impact: learning_impact.reasoning,
            });
        }
        
        self.save_db();
    }
}
