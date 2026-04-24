use crate::domain::entities::{
    CorrelatedRisk, CorrelationReport, RiskSeverity, ScanEvent, ScannedPage,
};
use chrono::Utc;

pub fn analyze_target(
    pages: &[ScannedPage],
    events: &mut Vec<ScanEvent>,
    tx: &Option<tokio::sync::mpsc::UnboundedSender<ScanEvent>>,
) -> CorrelationReport {
    let mut correlated_risks = Vec::new();

    let mut emit_event =
        |event_type: &str, level: &str, message: String, payload: Option<serde_json::Value>| {
            let ev = ScanEvent {
                timestamp: Utc::now(),
                event_type: event_type.to_string(),
                level: level.to_string(),
                message,
                payload,
            };
            events.push(ev.clone());
            if let Some(ref t) = tx {
                let _ = t.send(ev);
            }
        };

    emit_event(
        "CORRELATION_STARTED",
        "INFO",
        "Initiating Phase 5 Intelligent Correlation analysis".to_string(),
        None,
    );

    let sensitive_keywords = ["login", "admin", "auth", "account", "dashboard", "portal"];
    let total_pages = pages.len();

    let mut xfo_missing_count = 0;
    let mut csp_missing_count = 0;
    let mut missing_xfo_evidences = Vec::new();
    let mut missing_csp_evidences = Vec::new();

    let mut tech_server_combos = Vec::new();

    for page in pages {
        let url_lower = page.url.to_string().to_lowercase();
        let is_sensitive = sensitive_keywords.iter().any(|k| url_lower.contains(k));

        let mut has_hsts = false;
        let mut has_csp = false;
        let mut has_xfo = false;

        for hdr in &page.security_headers {
            if hdr.name == "Strict-Transport-Security"
                && hdr.status != crate::domain::entities::SecurityHeaderStatus::Missing
            {
                has_hsts = true;
            }
            if hdr.name == "Content-Security-Policy"
                && hdr.status != crate::domain::entities::SecurityHeaderStatus::Missing
            {
                has_csp = true;
            }
            if hdr.name == "X-Frame-Options"
                && hdr.status != crate::domain::entities::SecurityHeaderStatus::Missing
            {
                has_xfo = true;
            }
        }

        // 1. Sensitive Path Vulnerabilities
        if is_sensitive {
            if page.url.starts_with("https") && !has_hsts {
                let risk = CorrelatedRisk {
                    title: "Missing HSTS on Sensitive Path".to_string(),
                    severity: RiskSeverity::High,
                    explanation: "A sensitive authentication or administrative path is missing Strict-Transport-Security. This exposes the user session to man-in-the-middle downgrade attacks.".to_string(),
                    evidences: vec![format!("Detected on: {}", page.url)],
                };
                emit_event(
                    "RISK_GENERATED",
                    "WARN",
                    format!("Correlated Risk: {}", risk.title),
                    Some(serde_json::to_value(&risk).unwrap_or(serde_json::Value::Null)),
                );
                correlated_risks.push(risk);
            }

            if !has_csp {
                let risk = CorrelatedRisk {
                    title: "Missing CSP on Sensitive Path".to_string(),
                    severity: RiskSeverity::Medium,
                    explanation: "A sensitive path lacks a Content-Security-Policy. This significantly increases the impact of any Cross-Site Scripting (XSS) vulnerability, leading to account takeover.".to_string(),
                    evidences: vec![format!("Detected on: {}", page.url)],
                };
                emit_event(
                    "RISK_GENERATED",
                    "WARN",
                    format!("Correlated Risk: {}", risk.title),
                    Some(serde_json::to_value(&risk).unwrap_or(serde_json::Value::Null)),
                );
                correlated_risks.push(risk);
            }
        }

        // Accumulate for systemic checks
        if !has_xfo {
            xfo_missing_count += 1;
            if missing_xfo_evidences.len() < 3 {
                missing_xfo_evidences.push(page.url.clone());
            }
        }

        if !has_csp {
            csp_missing_count += 1;
            if missing_csp_evidences.len() < 3 {
                missing_csp_evidences.push(page.url.clone());
            }
        }

        // Record Tech + Server
        let server_hdr = page.headers.get("server").cloned();
        let powered_by = page.headers.get("x-powered-by").cloned();
        if server_hdr.is_some() || powered_by.is_some() {
            for tech in &page.detected_technologies {
                if tech.confidence_score > 0.6 {
                    tech_server_combos.push((
                        page.url.clone(),
                        tech.name.clone(),
                        server_hdr.clone(),
                        powered_by.clone(),
                    ));
                }
            }
        }
    }

    // 2. Systemic Misconfigurations (If 80% or more pages lack protection)
    if total_pages >= 3 {
        let threshold = (total_pages as f32 * 0.8).ceil() as usize;

        if xfo_missing_count >= threshold {
            let mut evidences = missing_xfo_evidences.clone();
            evidences.push(format!(
                "...and {} other pages",
                xfo_missing_count.saturating_sub(3)
            ));

            let risk = CorrelatedRisk {
                title: "Systemic Lack of Clickjacking Protection".to_string(),
                severity: RiskSeverity::Medium,
                explanation: format!("{} out of {} crawled pages completely lack X-Frame-Options or valid CSP frame-ancestors, exposing the application to widespread clickjacking.", xfo_missing_count, total_pages),
                evidences,
            };
            emit_event(
                "RISK_GENERATED",
                "WARN",
                format!("Correlated Risk: {}", risk.title),
                Some(serde_json::to_value(&risk).unwrap_or(serde_json::Value::Null)),
            );
            correlated_risks.push(risk);
        }

        if csp_missing_count >= threshold {
            let mut evidences = missing_csp_evidences.clone();
            evidences.push(format!(
                "...and {} other pages",
                csp_missing_count.saturating_sub(3)
            ));

            let risk = CorrelatedRisk {
                title: "Systemic Lack of Content-Security-Policy".to_string(),
                severity: RiskSeverity::Medium,
                explanation: format!("{} out of {} crawled pages do not implement CSP. Any XSS flaw discovered could immediately escalate to full client-side execution.", csp_missing_count, total_pages),
                evidences,
            };
            emit_event(
                "RISK_GENERATED",
                "WARN",
                format!("Correlated Risk: {}", risk.title),
                Some(serde_json::to_value(&risk).unwrap_or(serde_json::Value::Null)),
            );
            correlated_risks.push(risk);
        }
    }

    // 3. Technology Context Vector
    for (url, tech_name, server, powered) in tech_server_combos {
        let is_verbose_server = server.as_ref().is_some_and(|s| {
            s.contains('/')
                && (s.contains("Ubuntu")
                    || s.contains("Debian")
                    || s.contains("CentOS")
                    || s.chars().filter(|c| c.is_ascii_digit()).count() > 2)
        });
        let is_verbose_powered = powered.as_ref().is_some_and(|p| {
            p.contains('/') && p.chars().filter(|c| c.is_ascii_digit()).count() > 1
        });

        if is_verbose_server || is_verbose_powered {
            let mut ev = Vec::new();
            if let Some(s) = server {
                ev.push(format!("Server: {}", s));
            }
            if let Some(p) = powered {
                ev.push(format!("X-Powered-By: {}", p));
            }
            ev.push(format!("Detected Framework: {}", tech_name));
            ev.push(format!("Observed on: {}", url));

            let title = format!("Verbose Targeting Output ({})", tech_name);
            if !correlated_risks.iter().any(|r| r.title == title) {
                let risk = CorrelatedRisk {
                    title,
                    severity: RiskSeverity::Low,
                    explanation: "The target broadcasts exact software versions alongside identifying the underlying stack. This drastically reduces the time an attacker needs to find specific CVEs.".to_string(),
                    evidences: ev,
                };
                emit_event(
                    "RISK_GENERATED",
                    "WARN",
                    format!("Correlated Risk: {}", risk.title),
                    Some(serde_json::to_value(&risk).unwrap_or(serde_json::Value::Null)),
                );
                correlated_risks.push(risk);
            }
        }
    }

    emit_event(
        "CORRELATION_COMPLETE",
        "INFO",
        format!(
            "Correlation Engine finished. Derived {} intelligent risks.",
            correlated_risks.len()
        ),
        None,
    );

    // Calculate Overall Target Risk Score
    let mut score = 100i32;

    for page in pages {
        for risk in &page.risk_insights {
            match risk.severity {
                RiskSeverity::High => score -= 5,
                RiskSeverity::Medium => score -= 2,
                RiskSeverity::Low => score -= 1,
            }
        }
    }

    for risk in &correlated_risks {
        match risk.severity {
            RiskSeverity::High => score -= 25,
            RiskSeverity::Medium => score -= 15,
            RiskSeverity::Low => score -= 5,
        }
    }

    let overall_risk_score = score.clamp(0, 100) as u8;

    CorrelationReport {
        overall_risk_score,
        correlated_risks,
    }
}
