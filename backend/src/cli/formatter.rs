use anyhow::Result;

/// Output format enum used by clap for the `--format` flag.
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Sarif,
    Markdown,
}

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Counts total findings in the scan result.
pub fn count_findings(result: &serde_json::Value) -> usize {
    result["normalized_audit"]["findings"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0)
}

/// Counts findings matching a specific severity level.
pub fn count_by_severity(result: &serde_json::Value, severity: &str) -> usize {
    result["normalized_audit"]["findings"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|f| {
                    f["severity"]
                        .as_str()
                        .map(|s| s.eq_ignore_ascii_case(severity))
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Extracts the overall health score (0-100).
pub fn extract_score(result: &serde_json::Value) -> i64 {
    result["overall_health_score"]
        .as_f64()
        .map(|s| s as i64)
        .unwrap_or(0)
}

/// Extracts discovered endpoints from the scan result.
fn extract_endpoints(result: &serde_json::Value) -> Vec<serde_json::Value> {
    result["api_discovery"]["detected_endpoints"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// Generates the report URL using `LIMMA_REPORT_BASE_URL` env var (default: `https://limma.io/reports`).
fn generate_report_url(result: &serde_json::Value) -> String {
    let base = std::env::var("LIMMA_REPORT_BASE_URL")
        .unwrap_or_else(|_| "https://limma.io/reports".to_string());
    let scan_id = result["scan_id"].as_str().unwrap_or("unknown");
    format!("{}/{}", base, scan_id)
}

// ─── JSON Formatter ─────────────────────────────────────────────────────────

/// Formats the scan result as a CI-friendly JSON document.
pub fn format_json(result: &serde_json::Value) -> Result<String> {
    let formatted = serde_json::json!({
        "scan_id": result["scan_id"].as_str().unwrap_or("unknown"),
        "target": result["url"].as_str().unwrap_or("unknown"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "security_score": extract_score(result),
        "findings_count": count_findings(result),
        "p1_count": count_by_severity(result, "Critical"),
        "p2_count": count_by_severity(result, "High"),
        "p3_count": count_by_severity(result, "Medium"),
        "p4_count": count_by_severity(result, "Low"),
        "new_endpoints": extract_endpoints(result),
        "report_url": generate_report_url(result),
        "delta_report": result.get("delta_analysis").cloned().unwrap_or(serde_json::Value::Null),
        "raw_result": result,
    });

    Ok(serde_json::to_string_pretty(&formatted)?)
}

// ─── SARIF Formatter ────────────────────────────────────────────────────────

/// Formats the scan result as a SARIF 2.1.0 document.
/// This can be uploaded to GitHub Advanced Security or other SARIF-compatible tools.
pub fn format_sarif(result: &serde_json::Value) -> Result<String> {
    let sarif_results = convert_to_sarif_results(result);

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Limma Security Recon",
                    "informationUri": "https://limma.io",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": build_sarif_rules(result)
                }
            },
            "results": sarif_results
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Converts findings to SARIF result objects.
fn convert_to_sarif_results(result: &serde_json::Value) -> Vec<serde_json::Value> {
    let findings = match result["normalized_audit"]["findings"].as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    findings
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let severity = f["severity"].as_str().unwrap_or("Medium");
            let sarif_level = match severity {
                "Critical" | "High" => "error",
                "Medium" => "warning",
                _ => "note",
            };

            serde_json::json!({
                "ruleId": f["canonical_slug"].as_str()
                    .unwrap_or(&format!("limma-finding-{}", i)),
                "level": sarif_level,
                "message": {
                    "text": f["title"].as_str().unwrap_or("Security finding detected")
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": result["url"].as_str().unwrap_or("unknown")
                        }
                    }
                }],
                "properties": {
                    "security-severity": match severity {
                        "Critical" => "9.5",
                        "High" => "7.5",
                        "Medium" => "5.0",
                        "Low" => "2.5",
                        _ => "1.0",
                    },
                    "limma-confidence": f["confidence"].as_str().unwrap_or("tentative"),
                    "limma-severity": severity,
                }
            })
        })
        .collect()
}

/// Builds SARIF rule descriptors from the findings.
fn build_sarif_rules(result: &serde_json::Value) -> Vec<serde_json::Value> {
    let findings = match result["normalized_audit"]["findings"].as_array() {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    let mut seen = std::collections::HashSet::new();
    findings
        .iter()
        .filter_map(|f| {
            let slug = f["canonical_slug"].as_str()?;
            if seen.contains(slug) {
                return None;
            }
            seen.insert(slug.to_string());
            Some(serde_json::json!({
                "id": slug,
                "shortDescription": {
                    "text": f["title"].as_str().unwrap_or(slug)
                },
                "fullDescription": {
                    "text": f["description"].as_str().unwrap_or("")
                },
                "defaultConfiguration": {
                    "level": match f["severity"].as_str().unwrap_or("Medium") {
                        "Critical" | "High" => "error",
                        "Medium" => "warning",
                        _ => "note",
                    }
                }
            }))
        })
        .collect()
}

// ─── Markdown Formatter ─────────────────────────────────────────────────────

/// Formats the scan result as a human-readable Markdown report.
pub fn format_markdown(result: &serde_json::Value) -> Result<String> {
    let target = result["url"].as_str().unwrap_or("unknown");
    let score = extract_score(result);
    let total = count_findings(result);
    let p1 = count_by_severity(result, "Critical");
    let p2 = count_by_severity(result, "High");
    let p3 = count_by_severity(result, "Medium");
    let p4 = count_by_severity(result, "Low");
    let report_url = generate_report_url(result);

    let findings_section = format_findings_markdown(result);

    let md = format!(
        r#"# 🔍 Limma Security Scan Report

**Target:** `{target}`
**Scan Date:** {date}
**Security Score:** {score}/100

## Summary

| Metric | Count |
|--------|-------|
| **Total Findings** | {total} |
| **P1 (Critical)** | {p1} ⚠️ |
| **P2 (High)** | {p2} |
| **P3 (Medium)** | {p3} |
| **P4 (Low)** | {p4} |

## Findings

{findings_section}

---

[View Full Report]({report_url})

*Generated by [Limma Security Recon](https://limma.io) — {version}*
"#,
        date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        version = env!("CARGO_PKG_VERSION"),
    );

    Ok(md)
}

/// Renders individual findings as Markdown sections.
fn format_findings_markdown(result: &serde_json::Value) -> String {
    let findings = match result["normalized_audit"]["findings"].as_array() {
        Some(arr) => arr,
        None => return "*No findings detected.*".to_string(),
    };

    if findings.is_empty() {
        return "*No findings detected.*".to_string();
    }

    findings
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let severity = f["severity"].as_str().unwrap_or("Unknown");
            let icon = match severity {
                "Critical" => "🔴",
                "High" => "🟠",
                "Medium" => "🟡",
                "Low" => "🟢",
                _ => "⚪",
            };
            let title = f["title"].as_str().unwrap_or("Untitled Finding");
            let desc = f["description"].as_str().unwrap_or("");
            let remediation = f["remediation"]
                .as_str()
                .map(|r| format!("\n   > **Remediation:** {}", r))
                .unwrap_or_default();

            format!(
                "{}. {} **{}** — `{}`\n   {}{}\n",
                i + 1,
                icon,
                title,
                severity,
                desc,
                remediation
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
