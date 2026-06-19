use super::{EvidenceItem, EvidenceKind, EvidenceStrength};

pub struct TokenMatcher;

impl TokenMatcher {
    pub fn find_any(body: &str, indicators: &[&str]) -> Option<EvidenceItem> {
        indicators.iter().find_map(|indicator| {
            if body.contains(indicator) {
                Some(Self::evidence_for_indicator(indicator))
            } else {
                None
            }
        })
    }

    pub fn find_expected(body: &str, indicator: &str) -> Option<EvidenceItem> {
        if body.contains(indicator) {
            Some(Self::evidence_for_indicator(indicator))
        } else {
            None
        }
    }

    fn evidence_for_indicator(indicator: &str) -> EvidenceItem {
        let kind = if is_file_leak_token(indicator) {
            EvidenceKind::FileContent
        } else {
            EvidenceKind::TokenMatch
        };

        EvidenceItem::new(
            kind,
            EvidenceStrength::Conclusive,
            indicator,
            format!("Sensitive execution token matched: {}", indicator),
        )
    }
}

fn is_file_leak_token(indicator: &str) -> bool {
    matches!(
        indicator,
        "root:" | "root:x:0" | "[fonts]" | "[extensions]" | "/bin/bash"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_command_output_token() {
        let evidence = TokenMatcher::find_any("uid=33(www-data)", &["uid=", "root:"]);

        assert!(evidence.is_some());
        assert_eq!(evidence.unwrap().kind, EvidenceKind::TokenMatch);
    }
}
