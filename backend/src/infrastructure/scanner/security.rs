use std::collections::HashMap;
use crate::domain::entities::{
    SecurityHeaderResult, SecurityHeaderStatus, RiskInsight, RiskSeverity, DetectedTechnology,
};

pub fn audit_headers(headers: &HashMap<String, String>) -> Vec<SecurityHeaderResult> {
    let mut results = Vec::new();

    // Content-Security-Policy
    match headers.get("content-security-policy") {
        Some(val) => {
            let status = if val.contains("unsafe-inline") || val.contains("unsafe-eval") {
                SecurityHeaderStatus::Weak
            } else {
                SecurityHeaderStatus::Present
            };
            results.push(SecurityHeaderResult {
                name: "Content-Security-Policy".to_string(),
                status,
                value: Some(val.clone()),
                explanation: if status == SecurityHeaderStatus::Weak {
                    "CSP is present but allows unsafe execution (unsafe-inline / unsafe-eval).".to_string()
                } else {
                    "CSP is active, mitigating XSS risks.".to_string()
                },
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "Content-Security-Policy".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "Missing CSP. The site is vulnerable to cross-site scripting (XSS) and data injection attacks.".to_string(),
            });
        }
    }

    // Strict-Transport-Security
    match headers.get("strict-transport-security") {
        Some(val) => {
            let status = if val.contains("max-age=0") {
                SecurityHeaderStatus::Misconfigured
            } else if val.contains("max-age") {
                SecurityHeaderStatus::Present
            } else {
                SecurityHeaderStatus::Weak
            };
            results.push(SecurityHeaderResult {
                name: "Strict-Transport-Security".to_string(),
                status,
                value: Some(val.clone()),
                explanation: if status == SecurityHeaderStatus::Misconfigured {
                    "HSTS is disabled via max-age=0.".to_string()
                } else if status == SecurityHeaderStatus::Weak {
                    "HSTS lacks a valid max-age directive.".to_string()
                } else {
                    "HSTS is forcing secure connections.".to_string()
                },
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "Strict-Transport-Security".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "Missing HSTS. The site does not enforce HTTPS, allowing potential downgrade attacks.".to_string(),
            });
        }
    }

    // X-Frame-Options
    match headers.get("x-frame-options") {
        Some(val) => {
            let l_val = val.to_lowercase();
            let status = if l_val == "deny" || l_val == "sameorigin" {
                SecurityHeaderStatus::Present
            } else {
                SecurityHeaderStatus::Misconfigured
            };
            results.push(SecurityHeaderResult {
                name: "X-Frame-Options".to_string(),
                status,
                value: Some(val.clone()),
                explanation: if status == SecurityHeaderStatus::Misconfigured {
                    "XFO has an invalid or overly permissible value.".to_string()
                } else {
                    "XFO prevents clickjacking iframe rendering.".to_string()
                },
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "X-Frame-Options".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "Missing XFO. The page is vulnerable to clickjacking.".to_string(),
            });
        }
    }

    // X-Content-Type-Options
    match headers.get("x-content-type-options") {
        Some(val) => {
            let status = if val.to_lowercase() == "nosniff" {
                SecurityHeaderStatus::Present
            } else {
                SecurityHeaderStatus::Misconfigured
            };
            results.push(SecurityHeaderResult {
                name: "X-Content-Type-Options".to_string(),
                status,
                value: Some(val.clone()),
                explanation: if status == SecurityHeaderStatus::Misconfigured {
                    "Improper configurations found, expected 'nosniff'.".to_string()
                } else {
                    "Prevents MIME-sniffing vulnerabilities.".to_string()
                },
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "X-Content-Type-Options".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "Missing MIME sniffing protection.".to_string(),
            });
        }
    }

    // Referrer-Policy
    match headers.get("referrer-policy") {
        Some(val) => {
            let status = if val.contains("unsafe-url") {
                SecurityHeaderStatus::Weak
            } else {
                SecurityHeaderStatus::Present
            };
            results.push(SecurityHeaderResult {
                name: "Referrer-Policy".to_string(),
                status,
                value: Some(val.clone()),
                explanation: if status == SecurityHeaderStatus::Weak {
                    "Referrer policy is overly permissive and leaks the full URL.".to_string()
                } else {
                    "Referrer sharing is restricted appropriately.".to_string()
                },
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "Referrer-Policy".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "No strict Referrer-Policy, potentially leaking sensitive path information to third parties.".to_string(),
            });
        }
    }

    // Permissions-Policy
    match headers.get("permissions-policy") {
        Some(val) => {
            results.push(SecurityHeaderResult {
                name: "Permissions-Policy".to_string(),
                status: SecurityHeaderStatus::Present,
                value: Some(val.clone()),
                explanation: "Policy dictates how browser features/APIs can be used.".to_string(),
            });
        }
        None => {
            results.push(SecurityHeaderResult {
                name: "Permissions-Policy".to_string(),
                status: SecurityHeaderStatus::Missing,
                value: None,
                explanation: "Permissions-Policy is missing. Browsers may allow dangerous API usages (camera, geolocation).".to_string(),
            });
        }
    }

    // Access-Control-Allow-Origin
    if let Some(val) = headers.get("access-control-allow-origin") {
        let status = if val == "*" || val == "null" {
            SecurityHeaderStatus::Weak
        } else {
            SecurityHeaderStatus::Present
        };
        results.push(SecurityHeaderResult {
            name: "Access-Control-Allow-Origin".to_string(),
            status,
            value: Some(val.clone()),
            explanation: if status == SecurityHeaderStatus::Weak {
                "Permissive CORS detected, allowing access from any origin.".to_string()
            } else {
                "CORS origin is restricted.".to_string()
            },
        });
    }

    results
}

pub fn generate_insights(
    headers: &HashMap<String, String>,
    technologies: &[DetectedTechnology],
    final_url: &str,
) -> Vec<RiskInsight> {
    let mut insights = Vec::new();

    // 1. Missing HSTS on HTTPS
    if final_url.starts_with("https://") && !headers.contains_key("strict-transport-security") {
        insights.push(RiskInsight {
            title: "Missing HSTS on HTTPS Target".to_string(),
            severity: RiskSeverity::Medium,
            explanation: "The target is served over HTTPS but lacks Strict-Transport-Security (HSTS). This allows attackers to perform SSL-stripping and downgrade users to a cleartext HTTP connection.".to_string(),
            evidence: "Missing 'strict-transport-security' header.".to_string(),
        });
    }

    // 2. Permissive CORS
    if let Some(cors) = headers.get("access-control-allow-origin") {
        if cors == "*" || cors == "null" {
            let has_creds = headers.get("access-control-allow-credentials").map_or(false, |v| v.to_lowercase() == "true");
            let is_sensitive = ["api", "auth", "login", "admin"].iter().any(|&s| final_url.to_lowercase().contains(s));
            
            let severity = if has_creds || is_sensitive {
                RiskSeverity::High
            } else {
                RiskSeverity::Low
            };

            insights.push(RiskInsight {
                title: "Permissive CORS Policy".to_string(),
                severity: severity.clone(),
                explanation: if matches!(severity, RiskSeverity::High) {
                    "A wildcard (*) or 'null' is used for the allowable origin on a sensitive or credentialed endpoint. This enables cross-site data exfiltration.".to_string()
                } else {
                    "A wildcard (*) or 'null' is used for the allowable origin, but credentials are not permitted and no sensitive paths are exposed. This is a hygiene gap with low exploitability.".to_string()
                },
                evidence: format!("Access-Control-Allow-Origin: {}", cors),
            });
        }
    }

    // 3. Exposed Server Identity
    if let Some(server) = headers.get("server") {
        insights.push(RiskInsight {
            title: "Exposed Server Identity".to_string(),
            severity: RiskSeverity::Low,
            explanation: "The web server explicitly identifies its software and potentially its version. Attackers can cross-reference this with known vulnerabilities/CVEs.".to_string(),
            evidence: format!("Server: {}", server),
        });
    }
    if let Some(powered) = headers.get("x-powered-by") {
        insights.push(RiskInsight {
            title: "Exposed Framework/Stack".to_string(),
            severity: RiskSeverity::Medium,
            explanation: "The X-Powered-By header discloses the underlying application framework or language. This accelerates reconnaissance for an attacker.".to_string(),
            evidence: format!("X-Powered-By: {}", powered),
        });
    }

    // 4. Detected CMS Exposure
    for tech in technologies {
        if tech.category.to_lowercase().contains("cms") && tech.confidence_score > 0.7 {
            insights.push(RiskInsight {
                title: "CMS Exposure".to_string(),
                severity: RiskSeverity::Medium,
                explanation: format!("A Content Management System ({}) was detected with high confidence. Identifying the CMS allows attackers to run targeted exploit suites (like wpscan) to find vulnerable plugins/themes.", tech.name),
                evidence: format!("{} detected at {}% confidence.", tech.name, (tech.confidence_score * 100.0) as u32),
            });
        }
    }

    insights
}

pub fn calculate_security_score(headers: &[SecurityHeaderResult], insights: &[RiskInsight]) -> u8 {
    let mut score = 100i32;

    // Base deductions
    for h in headers {
        match h.status {
            SecurityHeaderStatus::Missing | SecurityHeaderStatus::Misconfigured => {
                let deduction = match h.name.as_str() {
                    "Content-Security-Policy" => 15,
                    "Strict-Transport-Security" => 15,
                    "X-Frame-Options" => 10,
                    "X-Content-Type-Options" => 5,
                    _ => 5,
                };
                score -= deduction;
            }
            SecurityHeaderStatus::Weak => score -= 5,
            _ => {}
        }
    }

    // Insight deductions
    for insight in insights {
        let deduction = match insight.severity {
            RiskSeverity::High => 20,
            RiskSeverity::Medium => 10,
            RiskSeverity::Low => 5,
        };
        score -= deduction;
    }

    if score < 0 {
        0
    } else if score > 100 {
        100
    } else {
        score as u8
    }
}
