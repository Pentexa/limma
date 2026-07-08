use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "redirect_01".into(),
            payload: "https://evil.com".into(),
            description: "Direct external URL redirect".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_02".into(),
            payload: "//evil.com".into(),
            description: "Protocol-relative URL redirect".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_03".into(),
            payload: "/\\evil.com".into(),
            description: "Backslash URL confusion".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_04".into(),
            payload: "https://evil.com%2f%2f".into(),
            description: "URL-encoded slash redirect bypass".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_05".into(),
            payload: "javascript:alert(document.domain)".into(),
            description: "JavaScript URI in redirect parameter".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("javascript:".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_06".into(),
            payload: "https://target.com@evil.com".into(),
            description: "URL authority confusion".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_07".into(),
            payload: "https://evil.com/".into(),
            description: "Direct external URL redirect with trailing slash".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_08".into(),
            payload: "https://target.com.evil.com".into(),
            description: "Subdomain/Domain confusion redirect".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("target.com.evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_09".into(),
            payload: "https://evil.com%23.target.com".into(),
            description: "Fragment bypass".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "redirect_10".into(),
            payload: "https://evil.com%3F.target.com".into(),
            description: "Query bypass".into(),
            expected_indicator: ExpectedIndicator::RedirectLocation("evil.com".into()),
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        }
    ]
}
