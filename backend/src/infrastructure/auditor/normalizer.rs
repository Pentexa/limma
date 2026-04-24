use crate::domain::entities::*;
use chrono::Utc;
use uuid::Uuid;

pub trait FindingNormalizer<T> {
    fn normalize(&self, target: &str, data: &T) -> Vec<SecurityAuditFinding>;
}

pub struct WebScannerNormalizer;
pub struct ServerInvestigatorNormalizer;
pub struct ApiDiscovererNormalizer;

/// Assigns severity based on the real-world impact of each security header.
/// CSP and HSTS are critical defense layers — their absence enables real attacks.
fn header_severity(name: &str, status: &SecurityHeaderStatus) -> SeverityLevel {
    let name_lower = name.to_lowercase();

    // Misconfigured is generally worse than Missing (active misconfiguration vs passive omission)
    let is_misconfigured = matches!(status, SecurityHeaderStatus::Misconfigured);
    let is_weak = matches!(status, SecurityHeaderStatus::Weak);

    if name_lower.contains("content-security-policy") {
        if is_weak {
            return SeverityLevel::Medium;
        } // unsafe-inline/unsafe-eval
        SeverityLevel::Medium // Missing CSP enables XSS
    } else if name_lower.contains("strict-transport-security") {
        if is_misconfigured {
            return SeverityLevel::Medium;
        } // max-age=0
        SeverityLevel::Medium // Missing HSTS enables SSL stripping
    } else if name_lower.contains("x-frame-options")
        || name_lower.contains("x-content-type-options")
    {
        SeverityLevel::Medium // Missing enables clickjacking or MIME sniffing attacks
    } else if name_lower.contains("access-control") {
        if is_weak {
            return SeverityLevel::High;
        } // Wildcard CORS
        SeverityLevel::Medium
    } else {
        // referrer-policy, permissions-policy, etc.
        SeverityLevel::Low
    }
}

/// Generates a context-aware real-world impact statement for security headers.
fn header_impact(header_name: &str) -> String {
    let name_lower = header_name.to_lowercase();
    let impact = if name_lower.contains("content-security-policy") || name_lower.contains("csp") {
        "Without CSP, browsers will execute any injected script. Attackers can perform Cross-Site Scripting (XSS) to steal session tokens, redirect users, or inject malicious payloads into the page."
    } else if name_lower.contains("strict-transport-security") || name_lower.contains("hsts") {
        "Without HSTS, the first connection to this domain may occur over plaintext HTTP, allowing man-in-the-middle attackers to intercept credentials, cookies, and session data before the TLS upgrade."
    } else if name_lower.contains("x-frame-options") {
        "Without X-Frame-Options, this page can be embedded in a malicious iframe. Attackers can overlay transparent elements to trick users into performing unintended actions (Clickjacking)."
    } else if name_lower.contains("x-content-type-options") {
        "Without X-Content-Type-Options, browsers may MIME-sniff responses and misinterpret uploaded files as executable scripts, enabling stored XSS via file upload vectors."
    } else if name_lower.contains("referrer-policy") {
        "Without a strict Referrer-Policy, sensitive URL parameters (tokens, session IDs, internal paths) leak to third-party origins through the Referer header on every outbound navigation."
    } else if name_lower.contains("permissions-policy") || name_lower.contains("feature-policy") {
        "Without Permissions-Policy, embedded third-party scripts can silently access powerful browser APIs (camera, microphone, geolocation), creating privacy and surveillance risks."
    } else if name_lower.contains("x-xss-protection") {
        "Without X-XSS-Protection, older browsers lose their built-in reflected XSS filter, making basic script injection attacks more likely to succeed."
    } else if name_lower.contains("access-control") || name_lower.contains("cors") {
        "A misconfigured CORS policy allows any malicious origin to read authenticated responses from this domain, enabling cross-site data exfiltration of user-specific content."
    } else {
        "The absence of this security header weakens the browser's defense posture, potentially exposing users to client-side attacks specific to this header's protection scope."
    };
    format!("\n\n[Real World Impact]: {}", impact)
}

/// Generates a context-aware real-world impact statement for risk insights.
fn risk_impact(title: &str, explanation: &str) -> String {
    let ctx = format!("{} {}", title, explanation).to_lowercase();
    let impact = if ctx.contains("cors")
        || ctx.contains("cross-origin")
        || ctx.contains("access-control")
    {
        "Any malicious website can issue cross-origin requests to this domain and read the response, allowing silent exfiltration of user data, tokens, and session state."
    } else if ctx.contains("cookie") || ctx.contains("set-cookie") {
        "Cookies transmitted without Secure/HttpOnly/SameSite flags can be intercepted over plaintext connections, accessed by injected JavaScript, or sent in cross-site request forgery attacks."
    } else if ctx.contains("server") || ctx.contains("version") || ctx.contains("x-powered-by") {
        "Exposing server software and version numbers allows attackers to search for known CVEs and exploit kits targeting this exact software build."
    } else if ctx.contains("redirect") || ctx.contains("301") || ctx.contains("302") {
        "Insecure or open redirects can be weaponized for phishing campaigns, where attackers use the trusted domain as a redirect proxy to land users on malicious pages."
    } else if ctx.contains("tls")
        || ctx.contains("ssl")
        || ctx.contains("certificate")
        || ctx.contains("https")
    {
        "Weak or misconfigured TLS allows passive network observers to decrypt traffic, exposing credentials, form submissions, and API tokens in transit."
    } else if ctx.contains("dns") || ctx.contains("nameserver") {
        "DNS misconfiguration can enable subdomain takeover, cache poisoning, or zone transfer attacks that give attackers control over traffic routing."
    } else if ctx.contains("technology") || ctx.contains("framework") || ctx.contains("cms") {
        "Identifying the technology stack narrows the attack surface for adversaries, enabling targeted exploitation of framework-specific vulnerabilities."
    } else {
        "This exposure provides reconnaissance value to attackers, reducing the effort required to craft targeted exploits against this infrastructure."
    };
    format!("\n\n[Real World Impact]: {}", impact)
}

/// Generates a context-aware real-world impact statement for server insights.
fn server_insight_impact(name: &str, explanation: &str) -> String {
    let ctx = format!("{} {}", name, explanation).to_lowercase();
    let impact = if ctx.contains("tls")
        || ctx.contains("ssl")
        || ctx.contains("cipher")
        || ctx.contains("certificate")
    {
        "Weak TLS configuration allows passive eavesdroppers on the same network (ISP, public WiFi) to decrypt the full TCP stream, exposing credentials and session data."
    } else if ctx.contains("http/2") || ctx.contains("http2") || ctx.contains("alpn") {
        "Lack of modern protocol support degrades performance and may expose the connection to protocol-downgrade attacks."
    } else if ctx.contains("dns") || ctx.contains("nameserver") || ctx.contains("resolve") {
        "DNS misconfiguration can enable subdomain takeover, cache poisoning, or zone transfer attacks compromising the entire domain."
    } else if ctx.contains("port") || ctx.contains("service") || ctx.contains("open") {
        "Unnecessary open ports expand the attack surface, potentially exposing internal services or debug interfaces to unauthorized access."
    } else if ctx.contains("header") || ctx.contains("server:") || ctx.contains("x-powered-by") {
        "Server banner disclosure enables version-specific exploit research, eliminating guesswork for attackers."
    } else {
        "This infrastructure signal provides attackers with reconnaissance data that accelerates targeted attack planning."
    };
    format!("\n\n[Real World Impact]: {}", impact)
}

impl FindingNormalizer<WebScanResult> for WebScannerNormalizer {
    fn normalize(&self, target: &str, data: &WebScanResult) -> Vec<SecurityAuditFinding> {
        let mut findings = Vec::new();

        for risk in &data.risk_insights {
            let severity = match risk.severity {
                RiskSeverity::High => SeverityLevel::High,
                RiskSeverity::Medium => SeverityLevel::Medium,
                RiskSeverity::Low => SeverityLevel::Low,
            };

            findings.push(SecurityAuditFinding {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                target_identifier: target.to_string(),
                affected_path_or_endpoint: None,
                protocol: Some("HTTP".to_string()),
                method: None,
                category: FindingCategory::InformationDisclosure,
                severity,
                confidence: ConfidenceLevel::Firm,
                status: FindingStatus::Open,
                summary: risk.title.clone(),
                technical_details: format!(
                    "{}{}",
                    risk.explanation,
                    risk_impact(&risk.title, &risk.explanation)
                ),
                source_module: SourceModule::WebScanner,
                evidence: vec![AuditEvidenceItem {
                    description: "Risk Evidence".to_string(),
                    validation_context: Some(
                        "Observed directly in the HTTP Response payload".to_string(),
                    ),
                    raw_data: risk.evidence.clone(),
                }],
                raw_reference: serde_json::to_value(risk).ok(),
                correlation_group_id: None,
                correlation_count: 0,
                correlation_type: None,
                correlation_confidence: None,
                correlation_summary: None,
                correlation_is_hygiene_gap: false,
                related_findings: Vec::new(),
                risk_score: None,
                exploitability: None,
                context_summary: None,
                evidence_weight: None,
                context_assessment: None,
                certainty: None,
            });
        }

        for header in &data.security_headers {
            if header.status == SecurityHeaderStatus::Missing
                || header.status == SecurityHeaderStatus::Weak
                || header.status == SecurityHeaderStatus::Misconfigured
            {
                // Severity based on real-world impact of each header
                let severity = header_severity(&header.name, &header.status);

                let summary = match header.status {
                    SecurityHeaderStatus::Missing => format!("Missing {}", header.name),
                    SecurityHeaderStatus::Weak => format!("Weak {}", header.name),
                    SecurityHeaderStatus::Misconfigured => format!("Misconfigured {}", header.name),
                    _ => format!("Insecure Header: {}", header.name),
                };

                findings.push(SecurityAuditFinding {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    target_identifier: target.to_string(),
                    affected_path_or_endpoint: None,
                    protocol: Some("HTTP".to_string()),
                    method: None,
                    category: FindingCategory::SecurityMisconfiguration,
                    severity,
                    confidence: ConfidenceLevel::Certain,
                    status: FindingStatus::Open,
                    summary,
                    technical_details: format!(
                        "{}{}",
                        header.explanation,
                        header_impact(&header.name)
                    ),
                    source_module: SourceModule::WebScanner,
                    evidence: vec![AuditEvidenceItem {
                        description: "Header Status".to_string(),
                        validation_context: Some(
                            "Static header inspection on base domain response".to_string(),
                        ),
                        raw_data: format!("{:?}", header.status),
                    }],
                    raw_reference: serde_json::to_value(header).ok(),
                    correlation_group_id: None,
                    correlation_count: 0,
                    correlation_type: None,
                    correlation_confidence: None,
                    correlation_summary: None,
                    correlation_is_hygiene_gap: false,
                    related_findings: Vec::new(),
                    risk_score: None,
                    exploitability: None,
                    context_summary: None,
                    evidence_weight: None,
                    context_assessment: None,
                    certainty: None,
                });
            }
        }

        findings
    }
}

impl FindingNormalizer<ServerInfo> for ServerInvestigatorNormalizer {
    fn normalize(&self, target: &str, data: &ServerInfo) -> Vec<SecurityAuditFinding> {
        let mut findings = Vec::new();

        for insight in &data.security_insights {
            // Only Critical and Warning are actionable findings.
            // "Informational" and "Secure" are observations — not vulnerabilities.
            let severity = match insight.status.as_str() {
                "Critical" => SeverityLevel::Critical,
                "Warning" => SeverityLevel::Medium,
                _ => continue,
            };

            findings.push(SecurityAuditFinding {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                target_identifier: target.to_string(),
                affected_path_or_endpoint: None,
                protocol: None,
                method: None,
                category: FindingCategory::InsecureTransport,
                severity,
                confidence: ConfidenceLevel::Firm,
                status: FindingStatus::Open,
                summary: insight.name.clone(),
                technical_details: format!(
                    "{}{}",
                    insight.explanation,
                    server_insight_impact(&insight.name, &insight.explanation)
                ),
                source_module: SourceModule::ServerInvestigator,
                evidence: vec![AuditEvidenceItem {
                    description: "Insight Evidence".to_string(),
                    validation_context: Some(
                        "Analyzed during active connection lifecycle".to_string(),
                    ),
                    raw_data: insight.evidence.clone(),
                }],
                raw_reference: serde_json::to_value(insight).ok(),
                correlation_group_id: None,
                correlation_count: 0,
                correlation_type: None,
                correlation_confidence: None,
                correlation_summary: None,
                correlation_is_hygiene_gap: false,
                related_findings: Vec::new(),
                risk_score: None,
                exploitability: None,
                context_summary: None,
                evidence_weight: None,
                context_assessment: None,
                certainty: None,
            });
        }

        for insight in &data.consistency_insights {
            findings.push(SecurityAuditFinding {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                target_identifier: target.to_string(),
                affected_path_or_endpoint: None,
                protocol: None,
                method: None,
                category: FindingCategory::SecurityMisconfiguration,
                severity: match insight.severity.as_str() {
                    "High" => SeverityLevel::High,
                    "Medium" => SeverityLevel::Medium,
                    "Low" => SeverityLevel::Low,
                    _ => SeverityLevel::Informational,
                },
                confidence: ConfidenceLevel::Firm,
                status: FindingStatus::Open,
                summary: insight.name.clone(),
                technical_details: format!(
                    "{}\n\n[Real World Impact]: Configuration inconsistencies across this server fleet reveal partial hardening. Attackers can identify and route traffic to the weakest node, bypassing protections enforced on other instances.",
                    insight.explanation
                ),
                source_module: SourceModule::ServerInvestigator,
                evidence: insight.evidences.iter().map(|e| AuditEvidenceItem {
                    description: "Inconsistency Detail".to_string(),
                    validation_context: Some("Correlated across multiple protocol responses".to_string()),
                    raw_data: e.clone(),
                }).collect(),
                raw_reference: serde_json::to_value(insight).ok(),
                correlation_group_id: None,
                correlation_count: 0,
                correlation_type: None,
                correlation_confidence: None,
                correlation_summary: None,
                correlation_is_hygiene_gap: false,
                related_findings: Vec::new(),
                risk_score: None,
                exploitability: None,
                context_summary: None,
                evidence_weight: None,
                context_assessment: None,
                certainty: None,
            });
        }

        findings
    }
}

impl FindingNormalizer<ApiDiscoveryResult> for ApiDiscovererNormalizer {
    fn normalize(&self, target: &str, data: &ApiDiscoveryResult) -> Vec<SecurityAuditFinding> {
        let mut findings = Vec::new();

        for endpoint in &data.detected_endpoints {
            let is_suspicious = endpoint.auth_likelihood == "None"
                && endpoint.confidence_score > 0.7
                && !endpoint.path.contains("public")
                && !endpoint.path.contains("health");

            if is_suspicious {
                findings.push(SecurityAuditFinding {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    target_identifier: target.to_string(),
                    affected_path_or_endpoint: Some(endpoint.path.clone()),
                    protocol: Some("HTTP".to_string()),
                    method: Some(endpoint.method_prediction.clone()),
                    category: FindingCategory::AuthenticationBypass,
                    severity: SeverityLevel::High,
                    confidence: ConfidenceLevel::Tentative,
                    status: FindingStatus::Open,
                    summary: format!("Potentially Unauthenticated API Endpoint: {}", endpoint.path),
                    technical_details: format!(
                        "Endpoint discovered with High confidence ({}) but lacking authentication constraints.\n\n[Real World Impact]: Any malicious origin or unauthorized entity can interact with this endpoint directly, bypass generic application logic, and potentially exfiltrate or modify sensitive data.",
                        endpoint.confidence_score
                    ),
                    source_module: SourceModule::ApiDiscoverer,
                    evidence: endpoint.evidences.iter().map(|e| AuditEvidenceItem {
                        description: format!("Source: {}", e.source_type),
                        validation_context: Some("Extracted via Javascript AST traversal and DOM state prediction".to_string()),
                        raw_data: e.snippet.clone(),
                    }).collect(),
                    raw_reference: serde_json::to_value(endpoint).ok(),
                    correlation_group_id: None,
                    correlation_count: 0,
                    correlation_type: None,
                    correlation_confidence: None,
                    correlation_summary: None,
                    correlation_is_hygiene_gap: false,
                    related_findings: Vec::new(),
                risk_score: None,
                exploitability: None,
                context_summary: None,
                evidence_weight: None,
                context_assessment: None,
                certainty: None,
                });
            }
        }

        findings
    }
}
