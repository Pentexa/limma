use serde::{Serialize, Deserialize};
use crate::domain::entities::{
    MasterReport, CanonicalFinding, SecurityAuditFinding,
    SeverityLevel, ConfidenceLevel, FindingCategory,
};

/// Nuclei template export format.
///
/// Converts Limma findings into Nuclei-compatible YAML templates
/// so users can re-verify findings with ProjectDiscovery's Nuclei scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleiExport {
    pub templates: Vec<NucleiTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleiTemplate {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub description: String,
    pub reference: Vec<String>,
    pub tags: Vec<String>,
    pub matchers: Vec<NucleiMatcher>,
    pub requests: Vec<NucleiRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleiMatcher {
    pub matcher_type: String, // "word", "regex", "status", "dsl"
    pub words: Vec<String>,
    pub part: String, // "header", "body", "status"
    pub condition: String, // "and", "or"
    pub negative: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleiRequest {
    pub method: String,
    pub path: Vec<String>,
    pub headers: std::collections::HashMap<String, String>,
}

impl NucleiExport {
    /// Build Nuclei templates from Limma MasterReport.
    pub fn from_master_report(report: &MasterReport) -> Self {
        let mut templates = Vec::new();

        if let Some(ref audit) = report.normalized_audit {
            for finding in &audit.canonical_findings {
                if let Some(template) = Self::finding_to_template(finding, &report.url) {
                    templates.push(template);
                }
            }
        }

        NucleiExport { templates }
    }

    fn finding_to_template(finding: &CanonicalFinding, target_url: &str) -> Option<NucleiTemplate> {
        let severity = severity_to_nuclei(&finding.severity);
        let id = format!("limma-{}", finding.canonical_slug.replace('/', "-").replace(' ', "-").to_lowercase());
        let tags = build_tags(finding);

        let (matchers, requests) = build_detection_logic(finding, target_url);

        // Only generate templates where we have meaningful detection logic
        if matchers.is_empty() {
            return None;
        }

        Some(NucleiTemplate {
            id,
            name: finding.title.clone(),
            severity,
            description: format!(
                "Limma detected: {}. Confidence: {}. Evidence count: {}.",
                finding.title,
                confidence_to_string(&finding.confidence),
                finding.merged_evidence_count
            ),
            reference: vec![
                "https://limma.io/docs".to_string(),
            ],
            tags,
            matchers,
            requests,
        })
    }

    /// Serialize all templates to a single YAML string (multi-document).
    pub fn to_yaml(&self) -> String {
        let mut output = String::new();

        for template in &self.templates {
            output.push_str("---\n");
            output.push_str(&format!("id: {}\n", template.id));
            output.push_str("\ninfo:\n");
            output.push_str(&format!("  name: {}\n", yaml_escape(&template.name)));
            output.push_str("  author: limma\n");
            output.push_str(&format!("  severity: {}\n", template.severity));
            output.push_str(&format!("  description: {}\n", yaml_escape(&template.description)));

            if !template.tags.is_empty() {
                output.push_str(&format!("  tags: {}\n", template.tags.join(",")));
            }

            if !template.reference.is_empty() {
                output.push_str("  reference:\n");
                for r in &template.reference {
                    output.push_str(&format!("    - {}\n", r));
                }
            }

            // HTTP requests
            output.push_str("\nhttp:\n");
            for req in &template.requests {
                output.push_str(&format!("  - method: {}\n", req.method));
                output.push_str("    path:\n");
                for p in &req.path {
                    output.push_str(&format!("      - \"{}\"\n", p));
                }

                if !req.headers.is_empty() {
                    output.push_str("    headers:\n");
                    for (k, v) in &req.headers {
                        output.push_str(&format!("      {}: \"{}\"\n", k, v));
                    }
                }
            }

            // Matchers
            if !template.matchers.is_empty() {
                output.push_str("    matchers:\n");
                for matcher in &template.matchers {
                    output.push_str(&format!("      - type: {}\n", matcher.matcher_type));
                    output.push_str(&format!("        part: {}\n", matcher.part));
                    if matcher.negative {
                        output.push_str("        negative: true\n");
                    }
                    if !matcher.words.is_empty() {
                        output.push_str("        words:\n");
                        for w in &matcher.words {
                            output.push_str(&format!("          - \"{}\"\n", w));
                        }
                    }
                    if matcher.condition != "or" {
                        output.push_str(&format!("        condition: {}\n", matcher.condition));
                    }
                }
            }

            output.push('\n');
        }

        output
    }
}

// ── Helper functions ──

fn severity_to_nuclei(severity: &SeverityLevel) -> String {
    match severity {
        SeverityLevel::Critical => "critical".to_string(),
        SeverityLevel::High => "high".to_string(),
        SeverityLevel::Medium => "medium".to_string(),
        SeverityLevel::Low => "low".to_string(),
        SeverityLevel::Informational => "info".to_string(),
    }
}

fn confidence_to_string(confidence: &ConfidenceLevel) -> String {
    match confidence {
        ConfidenceLevel::Certain => "certain".to_string(),
        ConfidenceLevel::Firm => "firm".to_string(),
        ConfidenceLevel::Tentative => "tentative".to_string(),
        ConfidenceLevel::Low => "low".to_string(),
    }
}

fn build_tags(finding: &CanonicalFinding) -> Vec<String> {
    let mut tags = vec!["limma".to_string(), "recon".to_string()];

    // Add category-based tags
    match &finding.risk_family {
        FindingCategory::SecurityMisconfiguration => tags.push("misconfig".to_string()),
        FindingCategory::InformationDisclosure => tags.push("exposure".to_string()),
        FindingCategory::InsecureTransport => tags.push("ssl".to_string()),
        FindingCategory::AuthenticationBypass => tags.push("auth".to_string()),
        FindingCategory::SuspiciousEndpoint => tags.push("exposure".to_string()),
        FindingCategory::InfrastructureLeak => tags.push("tech".to_string()),
        FindingCategory::Other(_) => tags.push("generic".to_string()),
    }

    // Add attack surface tags
    for tag in &finding.attack_surface_tags {
        let normalized = tag.to_lowercase().replace(' ', "-");
        if !tags.contains(&normalized) {
            tags.push(normalized);
        }
    }

    tags
}

fn build_detection_logic(
    finding: &CanonicalFinding,
    target_url: &str,
) -> (Vec<NucleiMatcher>, Vec<NucleiRequest>) {
    let mut matchers = Vec::new();
    let mut requests = Vec::new();

    // Determine paths to scan
    let paths: Vec<String> = if finding.affected_routes.is_empty() {
        vec!["{{BaseURL}}/".to_string()]
    } else {
        finding.affected_routes.iter()
            .map(|r| format!("{{{{BaseURL}}}}{}", r))
            .collect()
    };

    requests.push(NucleiRequest {
        method: "GET".to_string(),
        path: paths,
        headers: std::collections::HashMap::new(),
    });

    // Build matchers from underlying findings evidence
    for uf in &finding.underlying_findings {
        for ev in &uf.evidence {
            // Use evidence to create word matchers
            if !ev.raw_data.is_empty() && ev.raw_data.len() < 200 {
                let clean = ev.raw_data.trim().to_string();
                if !clean.is_empty() {
                    matchers.push(NucleiMatcher {
                        matcher_type: "word".to_string(),
                        words: vec![clean],
                        part: "header".to_string(),
                        condition: "or".to_string(),
                        negative: false,
                    });
                }
            }
        }
    }

    // For missing header findings, create negative matchers
    let slug = finding.canonical_slug.to_lowercase();
    if slug.contains("missing") && slug.contains("header") {
        let header_name = extract_header_name(&slug);
        if !header_name.is_empty() {
            matchers.push(NucleiMatcher {
                matcher_type: "word".to_string(),
                words: vec![header_name],
                part: "header".to_string(),
                condition: "or".to_string(),
                negative: true,
            });
        }
    }

    // Deduplicate matchers
    matchers.dedup_by(|a, b| a.words == b.words && a.part == b.part);

    (matchers, requests)
}

fn extract_header_name(slug: &str) -> String {
    // Try to extract header name from slugs like "missing-csp-header" or "missing-x-frame-options"
    let parts: Vec<&str> = slug.split('-').collect();
    if let Some(idx) = parts.iter().position(|&p| p == "missing") {
        let header_parts: Vec<&str> = parts[idx + 1..].iter()
            .filter(|&&p| p != "header")
            .copied()
            .collect();

        if !header_parts.is_empty() {
            return header_parts.join("-");
        }
    }
    String::new()
}

fn yaml_escape(s: &str) -> String {
    if s.contains(':') || s.contains('#') || s.contains('"') || s.contains('\'') || s.starts_with(' ') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}
