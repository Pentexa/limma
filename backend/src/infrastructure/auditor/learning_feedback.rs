use crate::domain::entities::{FeedbackAction, FeedbackEvent};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct FeedbackDB {
    pub events: Vec<FeedbackEvent>,
}

#[derive(Debug)]
pub struct LearningImpactResult {
    pub confidence_multiplier: f32, // Defaults to 1.0 (neutral)
    pub priority_modifier: i32,     // Numeric shift for Priority (-50 to +50)
    pub reasoning: Option<String>,
}

pub struct LearningFeedbackEngine {
    db_path: String,
    db: FeedbackDB,
}

impl LearningFeedbackEngine {
    pub fn new() -> Self {
        let db_path = "feedback_db.json".to_string();
        let db = Self::load_db(&db_path);
        Self { db_path, db }
    }

    fn load_db(path: &str) -> FeedbackDB {
        if Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(db) = serde_json::from_str(&content) {
                    return db;
                }
            }
        }
        FeedbackDB::default()
    }

    fn save_db(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.db) {
            let _ = fs::write(&self.db_path, content);
        }
    }

    pub fn record_feedback(&mut self, signature: String, action: FeedbackAction) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.db.events.push(FeedbackEvent {
            action,
            timestamp_sec: now,
            signature,
        });

        self.save_db();
    }

    pub fn generate_impact(&self, signature: &str) -> LearningImpactResult {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut tp_weight = 0.0;
        let mut fp_weight = 0.0;
        let mut ignored_weight = 0.0;
        let mut fixed_weight = 0.0;
        let mut relevant_events = 0;

        let half_life: f32 = 30.0 * 24.0 * 3600.0; // 30 days in seconds

        for event in &self.db.events {
            if event.signature == signature {
                relevant_events += 1;
                
                let age_sec = now.saturating_sub(event.timestamp_sec) as f32;
                // Exponential decay: e^(-lambda * t) where lambda = ln(2) / half_life
                let decay_factor = (-std::f32::consts::LN_2 * age_sec / half_life).exp();
                let effective_weight = 1.0 * decay_factor;

                match event.action {
                    FeedbackAction::VerifiedTruePositive => tp_weight += effective_weight,
                    FeedbackAction::FalsePositive => fp_weight += effective_weight,
                    FeedbackAction::Ignored => ignored_weight += effective_weight,
                    FeedbackAction::Fixed => fixed_weight += effective_weight,
                }
            }
        }

        if relevant_events == 0 {
            return LearningImpactResult {
                confidence_multiplier: 1.0,
                priority_modifier: 0,
                reasoning: None,
            };
        }

        // Calculate modifications
        let total_negative = fp_weight + (ignored_weight * 0.5); // Ignored is half as bad as FP
        let total_positive = tp_weight + (fixed_weight * 0.8);

        let mut confidence_multiplier = 1.0;
        let mut priority_modifier = 0;
        let mut reasoning = None;

        if total_negative > total_positive && total_negative > 1.5 {
            // Heavily penalized
            confidence_multiplier = 0.6; // Reduce confidence
            priority_modifier = -30; // Reduce priority
            reasoning = Some("Priority & Confidence reduced based on user feedback history (High FP/Ignore rate)".to_string());
        } else if total_positive > total_negative && total_positive > 1.0 {
            // Boosted
            confidence_multiplier = 1.3;
            priority_modifier = 20;
            reasoning = Some("Priority & Confidence boosted based on verified past actions".to_string());
        } else if fixed_weight > 0.0 {
            // It has been previously fixed, let's keep it visible
            priority_modifier = 10;
            reasoning = Some("Pattern has been actively fixed in the past; risk regression suspected".to_string());
        } else if fp_weight > 0.0 {
            // It has SOME false positive marks but not enough to drastically penalize yet
            confidence_multiplier = 0.8;
            priority_modifier = -10;
            reasoning = Some("Minor penalty: Pattern was marked as False Positive recently".to_string());
        }

        LearningImpactResult {
            confidence_multiplier,
            priority_modifier,
            reasoning,
        }
    }
}
