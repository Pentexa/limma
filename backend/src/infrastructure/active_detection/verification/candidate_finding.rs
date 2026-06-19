use uuid::Uuid;

use crate::domain::active_vuln::{ActiveVulnType, ExploitabilityLevel};
use crate::domain::entities::SeverityLevel;
use crate::infrastructure::active_detection::evidence::EvidenceItem;

#[derive(Debug, Clone)]
pub struct CandidateFinding {
    pub scan_id: Uuid,
    pub vuln_type: ActiveVulnType,
    pub target_url: String,
    pub affected_parameter: String,
    pub http_method: String,
    pub payload_used: String,
    pub request_raw: String,
    pub response_body: String,
    pub response_time_ms: u64,
    pub status_code: u16,
    pub severity: SeverityLevel,
    pub exploitability: ExploitabilityLevel,
    pub evidences: Vec<EvidenceItem>,
    pub expected_delay_ms: Option<u64>,
}

impl CandidateFinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scan_id: Uuid,
        vuln_type: ActiveVulnType,
        target_url: impl Into<String>,
        affected_parameter: impl Into<String>,
        http_method: impl Into<String>,
        payload_used: impl Into<String>,
        request_raw: impl Into<String>,
        response_body: impl Into<String>,
        response_time_ms: u64,
        status_code: u16,
        severity: SeverityLevel,
        exploitability: ExploitabilityLevel,
        evidences: Vec<EvidenceItem>,
    ) -> Self {
        Self {
            scan_id,
            vuln_type,
            target_url: target_url.into(),
            affected_parameter: affected_parameter.into(),
            http_method: http_method.into(),
            payload_used: payload_used.into(),
            request_raw: request_raw.into(),
            response_body: response_body.into(),
            response_time_ms,
            status_code,
            severity,
            exploitability,
            evidences,
            expected_delay_ms: None,
        }
    }

    pub fn with_expected_delay(mut self, expected_delay_ms: u64) -> Self {
        self.expected_delay_ms = Some(expected_delay_ms);
        self
    }

    pub fn matched_indicator(&self) -> String {
        self.evidences
            .iter()
            .map(|evidence| evidence.summary.clone())
            .collect::<Vec<_>>()
            .join("; ")
    }
}
