use crate::domain::entities::{
    ActivityEvent, ActivitySeverity, EvidenceItem, FingerprintMatch, HttpSummary, MatchStrength,
    TlsSummary,
};
use chrono::Utc;

use super::fingerprint_registry;
use super::signature_evaluator;

/// Evaluates all fingerprints from the registry against collected evidence.
/// Returns all matches sorted by confidence (best first), plus timeline events.
pub fn match_fingerprints(
    port: u16,
    evidence: &[EvidenceItem],
    tls_summary: &Option<TlsSummary>,
    http_summary: &Option<HttpSummary>,
    timeline: &mut Vec<ActivityEvent>,
) -> Vec<FingerprintMatch> {
    let registry = fingerprint_registry::load_registry();

    timeline.push(ActivityEvent {
        timestamp: Utc::now(),
        severity: ActivitySeverity::Info,
        event_type: "FP_EVAL_START".to_string(),
        message: format!(
            "[Port {}] Evaluating {} fingerprints against {} evidence items",
            port,
            registry.len(),
            evidence.len()
        ),
        metadata: None,
    });

    let mut all_matches: Vec<FingerprintMatch> = Vec::new();

    for fp in &registry {
        let result = signature_evaluator::evaluate_fingerprint(
            fp,
            port,
            evidence,
            tls_summary,
            http_summary,
        );

        // Log evaluation step
        let log_severity = match result.strength {
            MatchStrength::Full | MatchStrength::Strong => ActivitySeverity::Info,
            MatchStrength::Partial => ActivitySeverity::Info,
            MatchStrength::Weak | MatchStrength::NoMatch => ActivitySeverity::Info,
        };

        let should_log = result.strength != MatchStrength::NoMatch;

        if should_log {
            timeline.push(ActivityEvent {
                timestamp: Utc::now(),
                severity: log_severity,
                event_type: "FP_RULE_RESULT".to_string(),
                message: format!(
                    "[Port {}] {} → {:?} ({:.0}%, {}/{} rules)",
                    port,
                    fp.id,
                    result.strength,
                    result.confidence * 100.0,
                    result.matched_rules.len(),
                    result.matched_rules.len() + result.missing_rules.len(),
                ),
                metadata: None,
            });
        }

        // Only keep matches that meet minimum confidence
        if result.confidence >= fp.min_confidence && result.strength != MatchStrength::NoMatch {
            all_matches.push(result);
        }
    }

    // Sort candidates using Candidate Ranker logic
    // Tier priority: Specific > Generic > Fallback
    // Within same tier, sort by confidence descending.
    // A Generic fingerprint only beats a Specific one if it clearly outperforms (e.g., >20% confidence difference)
    all_matches.sort_by(|a, b| {
        let tier_score = |t: &crate::domain::entities::FingerprintTier| match t {
            crate::domain::entities::FingerprintTier::Specific => 2.0,
            crate::domain::entities::FingerprintTier::Generic => 1.0,
            crate::domain::entities::FingerprintTier::Fallback => 0.0,
        };

        let a_tier = tier_score(&a.tier);
        let b_tier = tier_score(&b.tier);

        // Calculate a ranking score: base confidence + tier bonus
        // A tier difference is worth roughly 0.20 confidence points
        let a_rank = a.confidence + (a_tier * 0.20);
        let b_rank = b.confidence + (b_tier * 0.20);

        b_rank
            .partial_cmp(&a_rank)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate: if two fingerprints point to the same service name, keep the best one
    let mut seen_services: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut deduped: Vec<FingerprintMatch> = Vec::new();
    for m in all_matches {
        if seen_services.insert(m.service_name.clone()) {
            deduped.push(m);
        }
    }

    if let Some(top) = deduped.first() {
        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "FP_TOP_MATCH".to_string(),
            message: format!(
                "[Port {}] Best: {} [{:?}] (Strength: {:?}, Conf: {:.0}%, Cov: {:?})",
                port,
                top.fingerprint_id,
                top.tier,
                top.strength,
                top.confidence * 100.0,
                top.coverage
            ),
            metadata: None,
        });
    } else {
        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Warning,
            event_type: "FP_NO_MATCH".to_string(),
            message: format!("[Port {}] No fingerprint matched above threshold", port),
            metadata: None,
        });
    }

    deduped
}
