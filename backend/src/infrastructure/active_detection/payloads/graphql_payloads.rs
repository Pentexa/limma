use crate::domain::active_vuln::{PayloadDefinition, ExpectedIndicator};
use crate::domain::entities::SeverityLevel;
use uuid::Uuid;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{ __schema { types { name fields { name } } } }".to_string(),
            description: "GraphQL: Introspection Query".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern("__schema".to_string()),
            severity: SeverityLevel::Low,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "query { a:__typename b:__typename c:__typename d:__typename e:__typename }".to_string(),
            description: "GraphQL: Query Batching/Alias DoS (Small)".to_string(),
            expected_indicator: ExpectedIndicator::StatusCode(200),
            severity: SeverityLevel::Medium,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "query { __schema { queryType { fields { type { fields { type { fields { type { name } } } } } } } } }".to_string(),
            description: "GraphQL: Deeply Nested Query DoS".to_string(),
            expected_indicator: ExpectedIndicator::TimeDelay(3000),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
    ]
}
