use crate::domain::active_vuln::ActiveVulnFinding;
use crate::domain::entities::ConfidenceLevel;
use crate::infrastructure::active_detection::differential::BaselineProfile;
use crate::infrastructure::active_detection::evidence::response_diff::ResponseDiffAnalyzer;
use crate::infrastructure::active_detection::evidence::{
    EvidenceItem, EvidenceKind, EvidenceStrength,
};
use crate::infrastructure::active_detection::timing::delay_analyzer::{
    DelayAnalysis, DelayAnalyzer,
};
use crate::infrastructure::active_detection::verification::confidence_engine::ConfidenceEngine;
use crate::infrastructure::active_detection::verification::finding_builder::FindingBuilder;
use crate::infrastructure::active_detection::verification::CandidateFinding;

pub struct VerificationPipeline;

impl VerificationPipeline {
    pub fn verify(
        candidate: CandidateFinding,
        baseline: Option<&BaselineProfile>,
    ) -> Option<ActiveVulnFinding> {
        let mut valid = false;
        let mut notes = candidate
            .evidences
            .iter()
            .map(|evidence| evidence.summary.clone())
            .collect::<Vec<_>>();
        let mut evidences = candidate.evidences.clone();
        let mut delay_analysis: Option<DelayAnalysis> = None;

        if baseline_contains_candidate_evidence(baseline, &candidate.evidences) {
            return None;
        }

        for evidence in &candidate.evidences {
            match evidence.kind {
                EvidenceKind::TokenMatch
                | EvidenceKind::FileContent
                | EvidenceKind::ErrorPattern
                | EvidenceKind::RedirectLocation
                | EvidenceKind::JwtAccepted => {
                    valid = true;
                }
                EvidenceKind::Reflection => {
                    if evidence.strength >= EvidenceStrength::Strong {
                        valid = true;
                    }
                }
                EvidenceKind::ResponseDiff => {
                    valid = true;
                }
                EvidenceKind::StatusCode => {
                    if let Some(baseline) = baseline {
                        let diff = ResponseDiffAnalyzer::compare_to_baseline(
                            baseline,
                            candidate.status_code,
                            &candidate.response_body,
                        );
                        if candidate.status_code != baseline.status_code || diff.is_significant() {
                            valid = true;
                            evidences.push(EvidenceItem::response_diff(
                                "baseline_response_diff",
                                format!(
                                    "Baseline comparison for status signal: {}",
                                    diff.summary()
                                ),
                            ));
                            notes.push(format!("Baseline comparison: {}", diff.summary()));
                        } else {
                            notes.push(
                                "Status-code signal matched baseline response; candidate rejected"
                                    .to_string(),
                            );
                        }
                    } else {
                        notes.push(
                            "Status-code-only candidate has no baseline profile; candidate rejected"
                                .to_string(),
                        );
                    }
                }
                EvidenceKind::TimeDelay => {
                    let expected_delay_ms = candidate.expected_delay_ms.unwrap_or(5_000);
                    let analysis = DelayAnalyzer::analyze(
                        baseline,
                        candidate.response_time_ms,
                        expected_delay_ms,
                    );
                    notes.push(analysis.summary());
                    if analysis.significant {
                        valid = true;
                    }
                    delay_analysis = Some(analysis);
                }
            }
        }

        if !valid {
            return None;
        }

        let confidence = ConfidenceEngine::calculate(
            &candidate,
            &evidences,
            delay_analysis.as_ref(),
            baseline.is_some(),
        );
        let verified = matches!(confidence, ConfidenceLevel::Certain | ConfidenceLevel::High);
        let matched_indicator = candidate.matched_indicator();

        Some(FindingBuilder::build(
            candidate,
            confidence,
            verified,
            matched_indicator,
            notes,
            false,
        ))
    }
}

fn baseline_contains_candidate_evidence(
    baseline: Option<&BaselineProfile>,
    evidences: &[EvidenceItem],
) -> bool {
    let Some(baseline) = baseline else {
        return false;
    };

    evidences.iter().any(|evidence| {
        matches!(
            evidence.kind,
            EvidenceKind::TokenMatch
                | EvidenceKind::FileContent
                | EvidenceKind::ErrorPattern
                | EvidenceKind::Reflection
        ) && !evidence.indicator.is_empty()
            && baseline.contains_indicator(&evidence.indicator)
    })
}
