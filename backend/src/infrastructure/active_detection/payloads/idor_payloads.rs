use crate::domain::active_vuln::{PayloadDefinition, ExpectedIndicator};
use crate::domain::entities::SeverityLevel;
use uuid::Uuid;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "0".to_string(),
            description: "IDOR: Boundary testing (ID = 0)".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "1".to_string(),
            description: "IDOR: Sequential ID test (ID = 1)".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "-1".to_string(),
            description: "IDOR: Negative boundary test (ID = -1)".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern("invalid input".to_string()), // Or checking if we get a DB error instead of 404/403
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "00000000-0000-0000-0000-000000000000".to_string(),
            description: "IDOR: Nil UUID test".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
    ]
}
