use crate::domain::entities::ConfidenceLevel;
use crate::infrastructure::active_detection::evidence::{
    EvidenceItem, EvidenceKind, EvidenceStrength,
};
use crate::infrastructure::active_detection::timing::delay_analyzer::DelayAnalysis;
use crate::infrastructure::active_detection::verification::CandidateFinding;

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn calculate(
        candidate: &CandidateFinding,
        evidences: &[EvidenceItem],
        delay_analysis: Option<&DelayAnalysis>,
        has_baseline: bool,
    ) -> ConfidenceLevel {
        if evidences.iter().any(|evidence| {
            matches!(
                evidence.kind,
                EvidenceKind::TokenMatch
                    | EvidenceKind::FileContent
                    | EvidenceKind::ErrorPattern
                    | EvidenceKind::RedirectLocation
            ) && evidence.strength == EvidenceStrength::Conclusive
        }) {
            return ConfidenceLevel::Certain;
        }

        if evidences
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::JwtAccepted)
        {
            return ConfidenceLevel::Firm;
        }

        if evidences.iter().any(|evidence| {
            evidence.kind == EvidenceKind::Reflection
                && evidence.strength == EvidenceStrength::Conclusive
        }) {
            return ConfidenceLevel::Certain;
        }

        if evidences.iter().any(|evidence| {
            evidence.kind == EvidenceKind::Reflection
                && evidence.strength >= EvidenceStrength::Strong
        }) {
            return ConfidenceLevel::Firm;
        }

        if evidences.iter().any(|evidence| {
            evidence.kind == EvidenceKind::ResponseDiff
                && evidence.strength >= EvidenceStrength::Strong
                && has_baseline
        }) {
            return ConfidenceLevel::Certain;
        }

        if evidences.iter().any(|evidence| {
            evidence.kind == EvidenceKind::ResponseDiff
                && evidence.strength == EvidenceStrength::Conclusive
        }) {
            return if has_baseline {
                ConfidenceLevel::Certain
            } else {
                ConfidenceLevel::Firm
            };
        }

        if let Some(delay_analysis) = delay_analysis {
            if delay_analysis.significant {
                return if has_baseline {
                    if delay_analysis.confidence_score >= 0.9 {
                        ConfidenceLevel::Certain
                    } else {
                        ConfidenceLevel::Firm
                    }
                } else {
                    ConfidenceLevel::Tentative
                };
            }
        }

        if evidences.iter().any(|evidence| {
            evidence.kind == EvidenceKind::StatusCode
                && evidence.strength >= EvidenceStrength::Medium
        }) {
            return if has_baseline {
                ConfidenceLevel::Firm
            } else {
                ConfidenceLevel::Tentative
            };
        }

        if candidate.expected_delay_ms.is_some() {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::Tentative
        }
    }
}
