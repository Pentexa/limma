use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;
use uuid::Uuid;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "${7*7}".to_string(),
            description: "SSTI: Freemarker / Velocity simple evaluation".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{{7*7}}".to_string(),
            description: "SSTI: Jinja2 / Twig simple evaluation".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "<% 7*7 %>".to_string(),
            description: "SSTI: ERB / Underscore simple evaluation".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{{config.items()}}".to_string(),
            description: "SSTI: Jinja2 config leak".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern("dict_items".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
    ]
}
