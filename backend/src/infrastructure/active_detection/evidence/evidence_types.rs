#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceStrength {
    Weak,
    Medium,
    Strong,
    Conclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    TokenMatch,
    Reflection,
    ErrorPattern,
    ResponseDiff,
    TimeDelay,
    StatusCode,
    FileContent,
    RedirectLocation,
    JwtAccepted,
}

#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub kind: EvidenceKind,
    pub strength: EvidenceStrength,
    pub indicator: String,
    pub summary: String,
}

impl EvidenceItem {
    pub fn new(
        kind: EvidenceKind,
        strength: EvidenceStrength,
        indicator: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            strength,
            indicator: indicator.into(),
            summary: summary.into(),
        }
    }

    pub fn token_match(indicator: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::TokenMatch,
            EvidenceStrength::Conclusive,
            indicator,
            summary,
        )
    }

    pub fn file_content(indicator: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::FileContent,
            EvidenceStrength::Conclusive,
            indicator,
            summary,
        )
    }

    pub fn error_pattern(indicator: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::ErrorPattern,
            EvidenceStrength::Conclusive,
            indicator,
            summary,
        )
    }

    pub fn response_diff(indicator: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::ResponseDiff,
            EvidenceStrength::Strong,
            indicator,
            summary,
        )
    }

    pub fn time_delay(observed_ms: u64, expected_delay_ms: u64) -> Self {
        Self::new(
            EvidenceKind::TimeDelay,
            EvidenceStrength::Strong,
            observed_ms.to_string(),
            format!(
                "Observed response delay: {}ms (expected injected delay: {}ms)",
                observed_ms, expected_delay_ms
            ),
        )
    }

    pub fn status_code(status_code: u16, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::StatusCode,
            EvidenceStrength::Weak,
            status_code.to_string(),
            summary,
        )
    }

    pub fn redirect_location(location: impl Into<String>, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::RedirectLocation,
            EvidenceStrength::Conclusive,
            location,
            summary,
        )
    }

    pub fn jwt_accepted(status_code: u16, summary: impl Into<String>) -> Self {
        Self::new(
            EvidenceKind::JwtAccepted,
            EvidenceStrength::Strong,
            status_code.to_string(),
            summary,
        )
    }
}
