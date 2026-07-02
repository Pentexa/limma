use crate::domain::entities::*;
use crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine;
use sha2::{Digest, Sha256};

pub struct AutonomousScanStrategyEngine {
    learning_engine: LearningFeedbackEngine,
}

impl AutonomousScanStrategyEngine {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            learning_engine: LearningFeedbackEngine::new(pool),
        }
    }

    pub async fn compute_strategy(
        &self,
        _analysis: &WebScanResult,
        api_discovery: &ApiDiscoveryResult,
        form_mapping: &FormMapping,
    ) -> Vec<ScanStrategyDecision> {
        let mut decisions = Vec::new();

        // 1. Analyze Auth Surfaces (Login forms)
        for form in &form_mapping.login_pages_found {
            decisions.push(ScanStrategyDecision {
                target: form.clone(),
                priority: TargetPriorityLevel::DeepAnalysis,
                adaptive_scan_depth: 3,
                reasoning: vec!["Deep scan triggered due to explicit auth exposure".to_string()],
            });
        }

        // 2. Discover API Endpoints Strategy
        for ep in &api_discovery.detected_endpoints {
            // Check learning loop to see if this endpoint has historically been a false positive or inert
            let canonical = format!(
                "{}|{:.3}",
                ep.path.trim().to_ascii_lowercase(),
                ep.auth_probability
            );
            let sig = format!("api_discovery:{:x}", Sha256::digest(canonical.as_bytes()));
            let learning_impact = self.learning_engine.generate_impact(&sig).await;

            let mut priority = TargetPriorityLevel::Standard;
            let mut depth = 2;
            let mut reasoning = vec!["Standard API surface discovered".to_string()];

            if ep.auth_probability > 0.7 {
                priority = TargetPriorityLevel::DeepAnalysis;
                depth = 4;
                reasoning.push("High auth probability detected on API route".to_string());
            }

            if learning_impact.priority_modifier < -10 {
                priority = TargetPriorityLevel::Deprioritized;
                depth = 1;
                reasoning.push(format!(
                    "Low-value surface deprioritized: {}",
                    learning_impact.reasoning.unwrap_or_default()
                ));
            }

            decisions.push(ScanStrategyDecision {
                target: ep.path.clone(),
                priority,
                adaptive_scan_depth: depth,
                reasoning,
            });
        }

        decisions
    }
}
