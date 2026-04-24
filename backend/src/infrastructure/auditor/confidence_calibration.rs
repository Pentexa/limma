use crate::domain::entities::*;
use sqlx::PgPool;

pub struct ConfidenceCalibrationEngine {
    pool: PgPool,
}

impl ConfidenceCalibrationEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn generate_signature(finding: &CanonicalFinding) -> String {
        let mut modules: Vec<String> = finding
            .contributing_modules
            .iter()
            .map(|s| format!("{:?}", s))
            .collect();
        modules.sort();
        format!("{}_[{}]", finding.canonical_slug, modules.join(","))
    }

    pub async fn update_from_scan(
        &mut self,
        canonical_findings: &mut [CanonicalFinding],
        learning_engine: &crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine,
    ) {
        for finding in canonical_findings.iter_mut() {
            let sig = Self::generate_signature(finding);

            // Fetch from DB
            let mut total_obs = 0;
            let mut succ = 0;
            let mut fail = 0;
            let mut partial = 0;
            let mut avg_rep = 0.0f32;

            use sqlx::Row;
            if let Ok(Some(row)) = sqlx::query(
                "SELECT total_observations, successful_verifications, failed_verifications, partial_verifications, average_reproducibility FROM confidence_calibration WHERE signature = $1"
            )
            .bind(&sig)
            .fetch_optional(&self.pool)
            .await
            {
                total_obs = row.try_get("total_observations").unwrap_or(0);
                succ = row.try_get("successful_verifications").unwrap_or(0);
                fail = row.try_get("failed_verifications").unwrap_or(0);
                partial = row.try_get("partial_verifications").unwrap_or(0);
                avg_rep = row.try_get::<f32, _>("average_reproducibility").unwrap_or(0.0);
            }

            if let Some(av) = &finding.active_verification {
                total_obs += 1;

                if av.status == VerificationStatus::VerifiedActionable {
                    succ += 1;
                } else if av.status == VerificationStatus::VerifiedInert {
                    fail += 1;
                } else if av.status == VerificationStatus::PartiallyVerified {
                    partial += 1;
                }

                let current_rep = av.reproducibility_score as f32;
                if total_obs == 1 {
                    avg_rep = current_rep;
                } else {
                    avg_rep =
                        ((avg_rep * (total_obs - 1) as f32) + current_rep) / (total_obs as f32);
                }

                let _ = sqlx::query(
                    "INSERT INTO confidence_calibration 
                    (signature, total_observations, successful_verifications, failed_verifications, partial_verifications, average_reproducibility) 
                    VALUES ($1, $2, $3, $4, $5, $6) 
                    ON CONFLICT(signature) DO UPDATE SET 
                    total_observations = EXCLUDED.total_observations,
                    successful_verifications = EXCLUDED.successful_verifications,
                    failed_verifications = EXCLUDED.failed_verifications,
                    partial_verifications = EXCLUDED.partial_verifications,
                    average_reproducibility = EXCLUDED.average_reproducibility"
                )
                .bind(&sig).bind(total_obs).bind(succ).bind(fail).bind(partial).bind(avg_rep)
                .execute(&self.pool)
                .await;
            }

            let mut reliability_coefficient = if total_obs > 0 {
                let rep_ratio = avg_rep / 100.0;
                0.1 + (rep_ratio * 1.4)
            } else {
                1.0
            };

            let original = finding.confidence.clone();
            let mut adjusted = original.clone();

            let mut impact = "Neutral - Insufficient History".to_string();
            let mut reasoning =
                "No historical calibration data exists for this pattern.".to_string();

            if total_obs >= 2 {
                if reliability_coefficient > 1.2 {
                    impact = "Confidence Boosted - Historically Reliable".to_string();
                    reasoning = format!("Pattern {} has a high empirical reproducibility score of {:.0}%. Confidence elevated.", sig, avg_rep);
                    adjusted = match original {
                        ConfidenceLevel::Tentative => ConfidenceLevel::Firm,
                        ConfidenceLevel::Firm => ConfidenceLevel::Certain,
                        _ => original.clone(),
                    }
                } else if reliability_coefficient < 0.6 {
                    impact = "Confidence Reduced - Historically Inconsistent".to_string();
                    reasoning = format!("Pattern {} frequently fails verification (Historical Rep.: {:.0}%). Confidence downgraded.", sig, avg_rep);
                    adjusted = match original {
                        ConfidenceLevel::Certain => ConfidenceLevel::Firm,
                        ConfidenceLevel::Firm => ConfidenceLevel::Tentative,
                        ConfidenceLevel::Tentative => ConfidenceLevel::Low,
                        _ => original.clone(),
                    }
                } else {
                    impact = "Confidence Maintained - Moderate History".to_string();
                    reasoning = format!("Pattern {} shows average reproducibility ({:.0}%). Maintaining baseline confidence.", sig, avg_rep);
                }
            }

            let learning_impact = learning_engine.generate_impact(&sig).await;
            reliability_coefficient *= learning_impact.confidence_multiplier;

            if learning_impact.confidence_multiplier < 0.9
                || learning_impact.confidence_multiplier > 1.1
            {
                impact = "Confidence Modified by User Feedback".to_string();
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
    }
}
