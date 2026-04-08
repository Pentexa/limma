use crate::domain::entities::{
    ChangeEvent, ChangeType, CollectorSnapshot, SnapshotDiff, PortProbeResult
};
use std::collections::HashMap;

pub fn compare(previous: &CollectorSnapshot, current: &CollectorSnapshot) -> SnapshotDiff {
    let mut changes: Vec<ChangeEvent> = Vec::new();
    let mut summaries: Vec<String> = Vec::new();

    let mut prev_ports: HashMap<u16, &PortProbeResult> = HashMap::new();
    for p in &previous.port_results {
        prev_ports.insert(p.port, p);
    }

    let mut curr_ports: HashMap<u16, &PortProbeResult> = HashMap::new();
    for p in &current.port_results {
        curr_ports.insert(p.port, p);
    }

    let mut added_ports = 0;
    let mut removed_ports = 0;
    let mut changed_services = 0;
    let mut changed_confidences = 0;

    for (port, curr_res) in &curr_ports {
        if let Some(prev_res) = prev_ports.get(port) {
            // Compare top candidates
            let prev_top = prev_res.service_candidates.first();
            let curr_top = curr_res.service_candidates.first();

            match (prev_top, curr_top) {
                (Some(prev), Some(curr)) => {
                    let mut is_changed = false;
                    
                    if prev.service_name != curr.service_name {
                        changes.push(ChangeEvent {
                            change_type: ChangeType::Changed,
                            resource: format!("Service on Port {}", port),
                            before: Some(prev.service_name.clone()),
                            after: Some(curr.service_name.clone()),
                            description: format!("Service changed from {} to {}", prev.service_name, curr.service_name),
                        });
                        changed_services += 1;
                        is_changed = true;
                    } else {
                        // Same service, check confidence
                        let conf_diff = (curr.confidence_breakdown.final_score - prev.confidence_breakdown.final_score).abs();
                        if conf_diff > 0.15 {
                            changes.push(ChangeEvent {
                                change_type: ChangeType::Changed,
                                resource: format!("Confidence for {} on Port {}", curr.service_name, port),
                                before: Some(format!("{:.0}%", prev.confidence_breakdown.final_score * 100.0)),
                                after: Some(format!("{:.0}%", curr.confidence_breakdown.final_score * 100.0)),
                                description: format!("Confidence shifted significantly"),
                            });
                            changed_confidences += 1;
                            is_changed = true;
                        }
                    }

                    if !is_changed {
                        changes.push(ChangeEvent {
                            change_type: ChangeType::Unchanged,
                            resource: format!("Port {}", port),
                            before: None,
                            after: None,
                            description: format!("No significant changes for {}", curr.service_name),
                        });
                    }
                },
                (None, Some(curr)) => {
                    changes.push(ChangeEvent {
                        change_type: ChangeType::Changed,
                        resource: format!("Port {}", port),
                        before: Some("No service identified".into()),
                        after: Some(curr.service_name.clone()),
                        description: format!("Service newly identified as {}", curr.service_name),
                    });
                    changed_services += 1;
                },
                (Some(prev), None) => {
                    changes.push(ChangeEvent {
                        change_type: ChangeType::Changed,
                        resource: format!("Port {}", port),
                        before: Some(prev.service_name.clone()),
                        after: Some("No service identified".into()),
                        description: format!("Lost identification of {}", prev.service_name),
                    });
                    changed_services += 1;
                },
                (None, None) => {
                    // Both have no top candidate, unchanged
                }
            }
        } else {
            // Newly opened port
            changes.push(ChangeEvent {
                change_type: ChangeType::Added,
                resource: format!("Port {}", port),
                before: None,
                after: Some(format!("{:?}", curr_res.state)),
                description: format!("New port discovered"),
            });
            added_ports += 1;
        }
    }

    for (port, _) in &prev_ports {
        if !curr_ports.contains_key(port) {
            changes.push(ChangeEvent {
                change_type: ChangeType::Removed,
                resource: format!("Port {}", port),
                before: Some("Open".into()),
                after: None,
                description: format!("Port is no longer open/responsive"),
            });
            removed_ports += 1;
        }
    }

    if added_ports > 0 { summaries.push(format!("{} new port(s) opened", added_ports)); }
    if removed_ports > 0 { summaries.push(format!("{} port(s) closed", removed_ports)); }
    if changed_services > 0 { summaries.push(format!("{} service identification(s) changed", changed_services)); }
    if changed_confidences > 0 { summaries.push(format!("{} significant confidence shift(s)", changed_confidences)); }

    if summaries.is_empty() {
        summaries.push("No significant changes detected".to_string());
    }

    SnapshotDiff {
        previous_timestamp: previous.timestamp,
        current_timestamp: current.timestamp,
        changes,
        summaries,
    }
}
