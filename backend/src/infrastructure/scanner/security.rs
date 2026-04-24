use crate::domain::entities::{
    DetectedTechnology, RiskInsight, RiskSeverity, SecurityHeaderResult, SecurityHeaderStatus,
};
use std::collections::HashMap;

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
                    "CSP is present but allows unsafe execution (unsafe-inline / unsafe-eval)."
                        .to_string()
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
            // Parse max-age value to catch zero-padded evasion (max-age=0000000)
            let max_age_value = val
                .to_lowercase()
                .split(';')
                .find(|s| s.trim().starts_with("max-age"))
                .and_then(|s| s.split('=').nth(1))
                .and_then(|s| s.trim().parse::<u64>().ok());

            let status = if let Some(age) = max_age_value {
                if age == 0 {
                    SecurityHeaderStatus::Misconfigured // max-age=0 or max-age=0000000
                } else {
                    SecurityHeaderStatus::Present
                }
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
    body: &str,
    redirect_chain: &[crate::domain::entities::RedirectChainEntry],
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

    // 2. Permissive CORS (Enhanced: Wildcard, Null, AND Domain Suffix Attacks)
    if let Some(cors) = headers.get("access-control-allow-origin") {
        let has_creds = headers
            .get("access-control-allow-credentials")
            .is_some_and(|v| v.to_lowercase() == "true");
        let is_sensitive = ["api", "auth", "login", "admin"]
            .iter()
            .any(|&s| final_url.to_lowercase().contains(s));

        if cors == "*" || cors == "null" {
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
        } else if has_creds {
            // Domain suffix attack detection: e.g. "trusted.com.evil.com"
            // Extract the origin's hostname and check if it looks suspicious
            if let Ok(origin_url) = url::Url::parse(cors) {
                if let Some(origin_host) = origin_url.host_str() {
                    // Check if the origin host has 3+ domain segments (possible subdomain of attacker apex)
                    let segments: Vec<&str> = origin_host.split('.').collect();
                    if segments.len() > 3 {
                        insights.push(RiskInsight {
                            title: "Suspicious CORS Origin — Possible Domain Suffix Attack".to_string(),
                            severity: RiskSeverity::High,
                            explanation: "The Access-Control-Allow-Origin header reflects a specific origin with credentials enabled, but the origin hostname has an unusually deep domain hierarchy. Attackers can register domains like 'trusted-app.company.evil.com' to bypass naive origin whitelists that use prefix matching instead of strict domain comparison.".to_string(),
                            evidence: format!("Access-Control-Allow-Origin: {} (with credentials enabled)", cors),
                        });
                    }
                }
            }
        }
    }

    // 3. Exposed Server Identity (Only if version is likely exposed)
    if let Some(server) = headers.get("server") {
        // Check for version pattern (e.g. nginx/1.18.0, Apache/2.4.49)
        let has_version = regex::Regex::new(r"\d+\.\d+").is_ok_and(|re| re.is_match(server));
        if has_version {
            insights.push(RiskInsight {
                title: "Exposed Server Identity".to_string(),
                severity: RiskSeverity::Medium,
                explanation: "The web server explicitly identifies its software version. Attackers can cross-reference this with known vulnerabilities/CVEs.".to_string(),
                evidence: format!("Server: {}", server),
            });
        }
    }
    if let Some(powered) = headers.get("x-powered-by") {
        insights.push(RiskInsight {
            title: "Exposed Framework/Stack".to_string(),
            severity: RiskSeverity::Medium,
            explanation: "The X-Powered-By header discloses the underlying application framework or language. This accelerates reconnaissance for an attacker.".to_string(),
            evidence: format!("X-Powered-By: {}", powered),
        });
    }

    // 3b. ASP.NET Specific Version Disclosures
    if let Some(aspnet_ver) = headers.get("x-aspnet-version") {
        insights.push(RiskInsight {
            title: "Exposed ASP.NET Runtime Version".to_string(),
            severity: RiskSeverity::Medium,
            explanation: "The X-AspNet-Version header reveals the exact .NET Framework runtime version. Attackers can cross-reference this with known CVEs targeting specific .NET builds (e.g., deserialization exploits, ViewState attacks).".to_string(),
            evidence: format!("X-AspNet-Version: {}", aspnet_ver),
        });
    }
    if let Some(mvc_ver) = headers.get("x-aspnetmvc-version") {
        insights.push(RiskInsight {
            title: "Exposed ASP.NET MVC Version".to_string(),
            severity: RiskSeverity::Medium,
            explanation: "The X-AspNetMvc-Version header reveals the specific MVC framework version. Combined with the runtime version, this gives attackers a precise fingerprint for targeted exploitation.".to_string(),
            evidence: format!("X-AspNetMvc-Version: {}", mvc_ver),
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

    // 5. HTML Comment Leakage Detection
    // Scan for sensitive information accidentally left in <!-- --> comments
    let comment_regex = regex::Regex::new(r"<!--([\s\S]*?)-->").unwrap();
    let sensitive_patterns: Vec<(&str, regex::Regex)> = vec![
        ("Password/Credential", regex::Regex::new(r"(?i)(password|passwd|pwd|secret|api[_-]?key|token|credential)\s*[:=]\s*\S+").unwrap()),
        ("Database Connection", regex::Regex::new(r"(?i)(db[_-]?pass|database|mysql|postgres|mongodb|connection[_-]?string|jdbc)").unwrap()),
        ("Internal IP Address", regex::Regex::new(r"(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3})").unwrap()),
        ("Server Version in Comment", regex::Regex::new(r"(?i)(apache|nginx|tomcat|iis|node|django|flask|rails|laravel|express)/[\d.]+").unwrap()),
        ("TODO/FIXME with Sensitive Context", regex::Regex::new(r"(?i)(todo|fixme|hack|xxx|bug)\s*:.*(?:auth|security|password|secret|vuln)").unwrap()),
    ];

    for cap in comment_regex.captures_iter(body) {
        let comment_text = cap.get(1).map_or("", |m| m.as_str());
        for (label, pattern) in &sensitive_patterns {
            if let Some(m) = pattern.find(comment_text) {
                // Truncate evidence to avoid leaking the actual secret in reports
                let evidence_snippet = if comment_text.len() > 120 {
                    format!("{}...", &comment_text[..120])
                } else {
                    comment_text.to_string()
                };
                insights.push(RiskInsight {
                    title: format!("Sensitive Data in HTML Comment — {}", label),
                    severity: RiskSeverity::High,
                    explanation: format!(
                        "An HTML comment contains what appears to be sensitive information ({}). \
                         Developers sometimes leave debug notes, credentials, or internal details \
                         in comments that are invisible to users but fully readable in the page source. \
                         Matched pattern: '{}'",
                        label, m.as_str()
                    ),
                    evidence: format!("<!-- {} -->", evidence_snippet.trim()),
                });
                break; // One finding per comment per pattern category is enough
            }
        }
    }

    // 6. Open Redirect Detection — analyze redirect chain for suspicious external destinations
    if redirect_chain.len() >= 2 {
        if let Ok(original_url) = url::Url::parse(&redirect_chain[0].url) {
            let orig_host = original_url.host_str().unwrap_or("").to_lowercase();
            for entry in &redirect_chain[1..] {
                if entry.status_code == 301
                    || entry.status_code == 302
                    || entry.status_code == 307
                    || entry.status_code == 308
                {
                    if let Ok(redirect_url) = url::Url::parse(&entry.url) {
                        let redirect_host = redirect_url.host_str().unwrap_or("").to_lowercase();
                        if !redirect_host.is_empty()
                            && redirect_host != orig_host
                            && !redirect_host.ends_with(&format!(".{}", orig_host))
                            && redirect_host != "localhost"
                            && !redirect_host.starts_with("127.")
                        {
                            insights.push(RiskInsight {
                                title: "Suspicious External Redirect Detected".to_string(),
                                severity: RiskSeverity::High,
                                explanation: format!(
                                    "The server redirects to an external domain '{}' which differs from the original target '{}'. \
                                     This may indicate an open redirect vulnerability that can be weaponized for phishing campaigns.",
                                    redirect_host, orig_host
                                ),
                                evidence: format!("HTTP {} → {}", entry.status_code, entry.url),
                            });
                        }
                    }
                }
            }
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
