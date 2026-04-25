use crate::domain::entities::FeedbackAction;
use sqlx::PgPool;

#[derive(Debug)]
pub struct LearningImpactResult {
    pub confidence_multiplier: f32, // Defaults to 1.0 (neutral)
    pub priority_modifier: i32,     // Numeric shift for Priority (-50 to +50)
    pub reasoning: Option<String>,
}

pub struct LearningFeedbackEngine {
    pool: PgPool,
}

impl LearningFeedbackEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_feedback(&self, signature: String, action: FeedbackAction) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let action_str = format!("{:?}", action);

        let _ = sqlx::query(
            "INSERT INTO learning_feedback (signature, action, timestamp_sec) VALUES ($1, $2, $3)",
        )
        .bind(signature)
        .bind(action_str)
        .bind(now)
        .execute(&self.pool)
        .await;
    }

    pub async fn generate_impact(&self, signature: &str) -> LearningImpactResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut tp_weight = 0.0;
        let mut fp_weight = 0.0;
        let mut ignored_weight = 0.0;
        let mut fixed_weight = 0.0;
        let mut relevant_events = 0;

        let half_life: f32 = 30.0 * 24.0 * 3600.0; // 30 days in seconds

        let records =
            sqlx::query("SELECT action, timestamp_sec FROM learning_feedback WHERE signature = $1")
                .bind(signature)
                .fetch_all(&self.pool)
                .await
                .unwrap_or_default();

        for record in records {
            use sqlx::Row;
            relevant_events += 1;

            let action: String = record.try_get("action").unwrap_or_default();
            let timestamp_sec: i64 = record.try_get("timestamp_sec").unwrap_or_default();
            let age_sec = (now - timestamp_sec).max(0) as f32;
            let decay_factor = (-std::f32::consts::LN_2 * age_sec / half_life).exp();
            let effective_weight = 1.0 * decay_factor;

            match action.as_str() {
                "VerifiedTruePositive" => tp_weight += effective_weight,
                "FalsePositive" => fp_weight += effective_weight,
                "Ignored" => ignored_weight += effective_weight,
                "Fixed" => fixed_weight += effective_weight,
                _ => {}
            }
        }

        if relevant_events == 0 {
            return LearningImpactResult {
                confidence_multiplier: 1.0,
                priority_modifier: 0,
                reasoning: None,
            };
        }

        let total_negative = fp_weight + (ignored_weight * 0.5);
        let total_positive = tp_weight + (fixed_weight * 0.8);

        let mut confidence_multiplier = 1.0;
        let mut priority_modifier = 0;
        let mut reasoning = None;

        if total_negative > total_positive && total_negative > 1.5 {
            confidence_multiplier = 0.6;
            priority_modifier = -30;
            reasoning = Some("Priority & Confidence reduced based on user feedback history (High FP/Ignore rate)".to_string());
        } else if total_positive > total_negative && total_positive > 1.0 {
            confidence_multiplier = 1.3;
            priority_modifier = 20;
            reasoning =
                Some("Priority & Confidence boosted based on verified past actions".to_string());
        } else if fixed_weight > 0.0 {
            priority_modifier = 10;
            reasoning = Some(
                "Pattern has been actively fixed in the past; risk regression suspected"
                    .to_string(),
            );
        } else if fp_weight > 0.0 {
            confidence_multiplier = 0.8;
            priority_modifier = -10;
            reasoning =
                Some("Minor penalty: Pattern was marked as False Positive recently".to_string());
        }

        LearningImpactResult {
            confidence_multiplier,
            priority_modifier,
            reasoning,
        }
    }
}
