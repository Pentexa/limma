use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;
use uuid::Uuid;

pub fn get_java_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "rO0ABXNyABFqYXZhLnV0aWwuSGFzaE1hcAAAAAAQPLLdAAwAAAAAAwB4".to_string(),
            description: "Java Deserialization: Base64 serialized empty HashMap".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "java.io.InvalidClassException".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "aced0005".to_string(),
            description: "Java Deserialization: Magic bytes (aced0005)".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "java.io.StreamCorruptedException".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_php_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "O:8:\"stdClass\":0:{}".to_string(),
            description: "PHP Deserialization: Empty stdClass object".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "PHP Notice:  unserialize()".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "a:1:{i:0;s:4:\"test\";}".to_string(),
            description: "PHP Deserialization: Simple array".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "unserialize(): Error at offset".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}

pub fn get_python_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "gASVKQAAAAAAAACMBXBvc2l4lIwGc3lzdGVtlJOUjBFlY2hvICdwaWNrbGVfdGVzdCculIWUUpQu"
                .to_string(),
            description: "Python Pickle: Base64 encoded echo command".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "pickle.UnpicklingError".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: Uuid::new_v4().to_string(),
            payload: "c__builtin__\neval\n(Vprint('pickle_test')\ntR.".to_string(),
            description: "Python Pickle: Plain eval".to_string(),
            expected_indicator: ExpectedIndicator::ErrorPattern(
                "pickle.UnpicklingError".to_string(),
            ),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
    ]
}
