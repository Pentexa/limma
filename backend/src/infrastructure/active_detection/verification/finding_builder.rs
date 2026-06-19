use chrono::Utc;
use uuid::Uuid;

use crate::domain::active_vuln::{ActiveVulnEvidence, ActiveVulnFinding};
use crate::domain::entities::ConfidenceLevel;
use crate::infrastructure::active_detection::verification::CandidateFinding;

pub struct FindingBuilder;

impl FindingBuilder {
    pub fn build(
        candidate: CandidateFinding,
        confidence: ConfidenceLevel,
        verified: bool,
        matched_indicator: String,
        additional_notes: Vec<String>,
        false_positive: bool,
    ) -> ActiveVulnFinding {
        ActiveVulnFinding {
            id: Uuid::new_v4(),
            scan_id: candidate.scan_id,
            timestamp: Utc::now(),
            vuln_type: candidate.vuln_type,
            target_url: candidate.target_url,
            affected_parameter: candidate.affected_parameter,
            http_method: candidate.http_method,
            payload_used: candidate.payload_used,
            evidence: ActiveVulnEvidence {
                request_raw: candidate.request_raw,
                response_raw: candidate.response_body.chars().take(2000).collect(),
                response_time_ms: candidate.response_time_ms,
                matched_indicator,
                additional_notes,
            },
            severity: candidate.severity,
            confidence,
            exploitability: candidate.exploitability,
            poc_generated: false,
            poc_id: None,
            verified,
            false_positive,
        }
    }
}
