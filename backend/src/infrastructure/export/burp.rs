use crate::domain::entities::{
    CanonicalFinding, ConfidenceLevel, MasterReport, SecurityAuditFinding, SeverityLevel,
};
use serde::{Deserialize, Serialize};

/// Burp Suite XML export format.
///
/// Converts Limma scan results into Burp Suite compatible project XML
/// so pentesters can import Limma recon directly into Burp for deep testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpExport {
    pub target: String,
    pub items: Vec<BurpItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurpItem {
    pub url: String,
    pub host: String,
    pub port: i32,
    pub protocol: String,
    pub method: String,
    pub path: String,
    pub extension: String,
    pub request: String,
    pub status_code: i32,
    pub response: String,
    pub comment: String,
    pub highlight: String,
}

impl BurpExport {
    /// Build a BurpExport from Limma MasterReport results.
    pub fn from_master_report(report: &MasterReport) -> Self {
        let mut items = Vec::new();

        let (host, port, protocol) = parse_target_url(&report.url);

        // Convert canonical findings to Burp items
        if let Some(ref audit) = report.normalized_audit {
            for finding in &audit.canonical_findings {
                let priority = severity_to_priority(&finding.severity);
                let highlight = severity_to_highlight(&finding.severity);

                for route in &finding.affected_routes {
                    items.push(BurpItem {
                        url: format!("{}://{}{}", protocol, host, route),
                        host: host.clone(),
                        port,
                        protocol: protocol.clone(),
                        method: "GET".to_string(),
                        path: route.clone(),
                        extension: extract_extension(route),
                        request: format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", route, host),
                        status_code: 0,
                        response: String::new(),
                        comment: format!(
                            "{}: {} ({})",
                            priority,
                            finding.title,
                            confidence_to_signal(&finding.confidence)
                        ),
                        highlight: highlight.clone(),
                    });
                }

                // If no routes, still create one item
                if finding.affected_routes.is_empty() {
                    items.push(BurpItem {
                        url: report.url.clone(),
                        host: host.clone(),
                        port,
                        protocol: protocol.clone(),
                        method: "GET".to_string(),
                        path: "/".to_string(),
                        extension: String::new(),
                        request: format!("GET / HTTP/1.1\r\nHost: {}\r\n\r\n", host),
                        status_code: 0,
                        response: String::new(),
                        comment: format!(
                            "{}: {} ({})",
                            priority,
                            finding.title,
                            confidence_to_signal(&finding.confidence)
                        ),
                        highlight: severity_to_highlight(&finding.severity),
                    });
                }
            }
        }

        // Convert API discovery endpoints
        if let Some(ref api) = report.api_discovery {
            for endpoint in &api.detected_endpoints {
                items.push(BurpItem {
                    url: format!("{}://{}{}", protocol, host, endpoint.path),
                    host: host.clone(),
                    port,
                    protocol: protocol.clone(),
                    method: endpoint.method_prediction.clone(),
                    path: endpoint.path.clone(),
                    extension: extract_extension(&endpoint.path),
                    request: format!(
                        "{} {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                        endpoint.method_prediction, endpoint.path, host
                    ),
                    status_code: endpoint
                        .runtime_verification
                        .as_ref()
                        .map(|rv| rv.status_code as i32)
                        .unwrap_or(0),
                    response: String::new(),
                    comment: format!(
                        "API Endpoint — Auth likelihood: {}, Confidence: {:.0}%",
                        endpoint.auth_likelihood,
                        endpoint.confidence_score * 100.0
                    ),
                    highlight: if endpoint.auth_probability < 0.3 {
                        "orange".to_string()
                    } else {
                        "blue".to_string()
                    },
                });
            }
        }

        BurpExport {
            target: report.url.clone(),
            items,
        }
    }

    /// Serialize to Burp Suite XML project format.
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\"?>\n");
        xml.push_str("<!DOCTYPE items [\n");
        xml.push_str("<!ELEMENT items (item*)>\n");
        xml.push_str("<!ATTLIST items burpVersion CDATA \"\">\n");
        xml.push_str("<!ELEMENT item (time, url, host, port, protocol, method, path, extension, request, status, responselength, mimetype, response, comment, highlight)>\n");
        xml.push_str("]>\n");
        xml.push_str("<items burpVersion=\"2024.0.0\">\n");

        for item in &self.items {
            xml.push_str("  <item>\n");
            xml.push_str(&format!(
                "    <time>{}</time>\n",
                chrono::Utc::now().format("%a %b %d %H:%M:%S %Z %Y")
            ));
            xml.push_str(&format!(
                "    <url><![CDATA[{}]]></url>\n",
                xml_escape(&item.url)
            ));
            xml.push_str(&format!(
                "    <host ip=\"\">{}</host>\n",
                xml_escape(&item.host)
            ));
            xml.push_str(&format!("    <port>{}</port>\n", item.port));
            xml.push_str(&format!(
                "    <protocol>{}</protocol>\n",
                xml_escape(&item.protocol)
            ));
            xml.push_str(&format!(
                "    <method><![CDATA[{}]]></method>\n",
                xml_escape(&item.method)
            ));
            xml.push_str(&format!(
                "    <path><![CDATA[{}]]></path>\n",
                xml_escape(&item.path)
            ));
            xml.push_str(&format!(
                "    <extension>{}</extension>\n",
                xml_escape(&item.extension)
            ));
            xml.push_str(&format!(
                "    <request base64=\"false\"><![CDATA[{}]]></request>\n",
                item.request
            ));
            xml.push_str(&format!("    <status>{}</status>\n", item.status_code));
            xml.push_str("    <responselength>0</responselength>\n");
            xml.push_str("    <mimetype></mimetype>\n");
            xml.push_str("    <response base64=\"false\"><![CDATA[]]></response>\n");
            xml.push_str(&format!(
                "    <comment><![CDATA[{}]]></comment>\n",
                item.comment
            ));
            xml.push_str(&format!("    <highlight>{}</highlight>\n", item.highlight));
            xml.push_str("  </item>\n");
        }

        xml.push_str("</items>\n");
        xml
    }
}

// ── Helper functions ──

fn parse_target_url(url: &str) -> (String, i32, String) {
    let parsed = url::Url::parse(url).unwrap_or_else(|_| {
        url::Url::parse(&format!("https://{}", url))
            .unwrap_or_else(|_| url::Url::parse("https://unknown").unwrap())
    });
    let host = parsed.host_str().unwrap_or("unknown").to_string();
    let protocol = parsed.scheme().to_string();
    let port = parsed
        .port()
        .unwrap_or(if protocol == "https" { 443 } else { 80 }) as i32;
    (host, port, protocol)
}

fn severity_to_priority(severity: &SeverityLevel) -> String {
    match severity {
        SeverityLevel::Critical => "P1".to_string(),
        SeverityLevel::High => "P2".to_string(),
        SeverityLevel::Medium => "P3".to_string(),
        SeverityLevel::Low | SeverityLevel::Informational => "P4".to_string(),
    }
}

fn severity_to_highlight(severity: &SeverityLevel) -> String {
    match severity {
        SeverityLevel::Critical => "orange".to_string(),
        SeverityLevel::High => "yellow".to_string(),
        SeverityLevel::Medium => "blue".to_string(),
        SeverityLevel::Low | SeverityLevel::Informational => "gray".to_string(),
    }
}

fn confidence_to_signal(confidence: &ConfidenceLevel) -> String {
    match confidence {
        ConfidenceLevel::Certain => "confirmed".to_string(),
        ConfidenceLevel::Firm => "likely".to_string(),
        ConfidenceLevel::Tentative => "unconfirmed".to_string(),
        ConfidenceLevel::Low => "pattern-only".to_string(),
    }
}

fn extract_extension(path: &str) -> String {
    path.rsplit('.')
        .next()
        .filter(|ext| ext.len() <= 6 && !ext.contains('/'))
        .unwrap_or("")
        .to_string()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
