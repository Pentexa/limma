use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackAction {
    Confirm,
    FalsePositive,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFeedbackEntry {
    pub rule_id: String,
    pub target_url: String,
    pub user_id: String,
    pub action: FeedbackAction,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleReputationStats {
    pub total_feedback: usize,
    pub confirmed: usize,
    pub false_positives: usize,
    pub ignored: usize,
    pub reputation_score: f64, // 0.0 to 100.0, base 50.0
}

impl Default for RuleReputationStats {
    fn default() -> Self {
        Self {
            total_feedback: 0,
            confirmed: 0,
            false_positives: 0,
            ignored: 0,
            reputation_score: 50.0,
        }
    }
}

pub struct RuleFeedbackEngine {
    entries: RwLock<Vec<RuleFeedbackEntry>>,
}

impl RuleFeedbackEngine {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    pub fn record_feedback(
        &self,
        rule_id: String,
        target_url: String,
        user_id: String,
        action: FeedbackAction,
    ) {
        let mut w = self.entries.write().unwrap();
        w.push(RuleFeedbackEntry {
            rule_id,
            target_url,
            user_id,
            action,
            timestamp: Utc::now(),
        });
    }

    pub fn get_rule_stats(&self, rule_id: &str) -> RuleReputationStats {
        let r = self.entries.read().unwrap();
        let mut stats = RuleReputationStats::default();

        for entry in r.iter().filter(|e| e.rule_id == rule_id) {
            stats.total_feedback += 1;
            match entry.action {
                FeedbackAction::Confirm => stats.confirmed += 1,
                FeedbackAction::FalsePositive => stats.false_positives += 1,
                FeedbackAction::Ignore => stats.ignored += 1,
            }
        }

        // Calculate reputation score
        if stats.total_feedback > 0 {
            // Give confirmations positive weight, FPs negative weight. Ignore is neutral.
            let c = stats.confirmed as f64;
            let fp = stats.false_positives as f64;
            let total_actionable = c + fp;

            if total_actionable > 0.0 {
                let ratio = c / total_actionable; // 0.0 to 1.0
                stats.reputation_score = ratio * 100.0;
            } else {
                stats.reputation_score = 50.0; // neutral if only ignored
            }
        }

        stats
    }

    pub fn get_feedback_history(&self) -> Vec<RuleFeedbackEntry> {
        let r = self.entries.read().unwrap();
        r.clone()
    }
}
