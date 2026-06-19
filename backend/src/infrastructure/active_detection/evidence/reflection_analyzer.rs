use super::{EvidenceItem, EvidenceKind, EvidenceStrength};

#[derive(Debug, Clone)]
pub struct ReflectionAnalysis {
    pub reflected: bool,
    pub html_encoded: bool,
    pub dangerous_context: bool,
    pub evidence: Option<EvidenceItem>,
}

pub struct ReflectionAnalyzer;

impl ReflectionAnalyzer {
    pub fn analyze(body: &str, payload: &str) -> ReflectionAnalysis {
        let reflected = body.contains(payload);
        let encoded = payload.replace('<', "&lt;").replace('>', "&gt;");
        let html_encoded = !reflected && body.contains(&encoded);

        if !reflected {
            return ReflectionAnalysis {
                reflected: false,
                html_encoded,
                dangerous_context: false,
                evidence: None,
            };
        }

        let body_lower = body.to_lowercase();
        let payload_lower = payload.to_lowercase();
        let dangerous_indicators = [
            "<script",
            "onerror=",
            "onload=",
            "onclick=",
            "onfocus=",
            "onmouseover=",
            "ontoggle=",
            "javascript:",
        ];
        let dangerous_context = dangerous_indicators
            .iter()
            .any(|indicator| payload_lower.contains(indicator) || body_lower.contains(indicator));

        ReflectionAnalysis {
            reflected,
            html_encoded,
            dangerous_context,
            evidence: Some(EvidenceItem::new(
                EvidenceKind::Reflection,
                if dangerous_context {
                    EvidenceStrength::Strong
                } else {
                    EvidenceStrength::Medium
                },
                payload,
                "Payload reflected in response body",
            )),
        }
    }
}
