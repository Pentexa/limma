use crate::domain::entities::{ScanEvent, ScanSummary, ScannedPage};
use chrono::Utc;
use std::collections::HashMap;

pub fn analyze_consistency(
    pages: &mut [ScannedPage],
    events: &mut Vec<ScanEvent>,
    tx: &Option<tokio::sync::mpsc::UnboundedSender<ScanEvent>>,
) -> ScanSummary {
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

    let total_pages = pages.len() as u32;
    let mut total_latency = 0;

    // tech_name -> occurrences
    let mut tech_counts = HashMap::new();

    for page in pages.iter() {
        total_latency += page.latency_ms;
        for tech in &page.detected_technologies {
            *tech_counts.entry(tech.name.clone()).or_insert(0) += 1;
        }
    }

    let average_latency_ms = if total_pages > 0 {
        total_latency / (total_pages as u64)
    } else {
        0
    };

    // Calculate common technologies
    let mut counts_vec: Vec<(String, u32)> = tech_counts.into_iter().collect();
    counts_vec.sort_by_key(|b| std::cmp::Reverse(b.1));
    let common_technologies: Vec<String> = counts_vec.iter().map(|(n, _)| n.clone()).collect();

    // Boost confidence and log events
    for (tech_name, count) in &counts_vec {
        if *count > 1 {
            emit_event(
                "TECH_CONFIDENCE_BOOST",
                "INFO",
                format!(
                    "Consistency check: '{}' found on {} pages. Boosting confidence.",
                    tech_name, count
                ),
                None,
            );

            // Boost on the first page
            if let Some(main_page) = pages.first_mut() {
                if let Some(tech) = main_page
                    .detected_technologies
                    .iter_mut()
                    .find(|t| t.name == *tech_name)
                {
                    tech.confidence_score = (tech.confidence_score + 0.15).min(1.0);
                } else {
                    emit_event(
                        "TECH_ISOLATED",
                        "WARN",
                        format!("Technology '{}' isolated to subpages only.", tech_name),
                        None,
                    );
                }
            }
        }
    }

    // Checking for variation in headers
    if pages.len() > 1 {
        let first_server = pages[0].headers.get("server").cloned();
        for page in pages.iter().skip(1) {
            let this_server = page.headers.get("server").cloned();
            if this_server != first_server {
                emit_event(
                    "HEADER_DISCREPANCY",
                    "WARN",
                    format!(
                        "Header discrepancy: Server identity changed on {}",
                        page.url
                    ),
                    None,
                );
            }
        }
    }

    ScanSummary {
        total_pages,
        average_latency_ms,
        common_technologies,
    }
}
