pub mod banner_probe;
pub mod confidence_engine;
pub mod diff_engine;
pub mod fallback_router;
pub mod fingerprint_matcher;
pub mod fingerprint_registry;
pub mod greeting_probe;
pub mod history_store;
pub mod http_probe;
pub mod nmap_validator;
pub mod signature_evaluator;
pub mod tls_probe;

use crate::domain::entities::{
    ActivityEvent, ActivitySeverity, CollectorSnapshot, CollectorStatus, PortProbeResult,
    PortState, ResolvedTarget, TargetInput,
};
use crate::domain::repositories::ServiceCollector;
use async_trait::async_trait;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use tokio::net::lookup_host;
use url::Url;

pub struct HttpServiceCollector;

impl HttpServiceCollector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ServiceCollector for HttpServiceCollector {
    async fn collect(
        &self,
        url_str: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<CollectorSnapshot, String> {
        let mut timeline = Vec::new();
        let errors = Vec::new();

        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "TARGET_RECEIVED".to_string(),
            message: format!("Target received: {}", url_str),
            metadata: None,
        });

        // === 1. Normalize Target ===
        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => {
                let with_scheme = format!("https://{}", url_str);
                Url::parse(&with_scheme).map_err(|e| e.to_string())?
            }
        };

        let host = url.host_str().unwrap_or(url_str).to_string();
        let scheme = Some(url.scheme().to_string());
        let default_port = url.port_or_known_default().unwrap_or(443);

        let target_input = TargetInput {
            original_input: url_str.to_string(),
            normalized_url: url.to_string(),
            host: host.clone(),
            scheme,
            default_port,
        };

        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "TARGET_NORMALIZED".to_string(),
            message: format!(
                "Normalized to host: {} (default port: {})",
                host, default_port
            ),
            metadata: None,
        });

        // === 2. DNS Resolution ===
        let lookup_str = format!("{}:{}", host, default_port);
        let ips = match lookup_host(&lookup_str).await {
            Ok(addrs) => {
                let addrs_vec: Vec<_> = addrs.collect();
                if addrs_vec.is_empty() {
                    return Err("DNS resolution returned no addresses".to_string());
                }
                addrs_vec
                    .into_iter()
                    .map(|a| a.ip().to_string())
                    .collect::<Vec<String>>()
            }
            Err(e) => {
                let err_msg = format!("DNS resolution failed: {}", e);
                timeline.push(ActivityEvent {
                    timestamp: Utc::now(),
                    severity: ActivitySeverity::Error,
                    event_type: "DNS_FAILED".to_string(),
                    message: err_msg.clone(),
                    metadata: None,
                });
                return Err(err_msg);
            }
        };

        let primary_ip = ips.first().cloned();

        let resolved_target = ResolvedTarget {
            ip_addresses: ips.clone(),
            primary_ip: primary_ip.clone(),
            hostname: Some(host.clone()),
        };

        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "DNS_RESOLVED".to_string(),
            message: format!("Resolved to {} IP(s): {}", ips.len(), ips.join(", ")),
            metadata: None,
        });

        // === 3. Port Probing (from EngineConfig) ===
        let target_ports = &profile.target_ports;

        let target_ip = primary_ip.ok_or("No primary IP resolved")?;

        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "SCAN_STARTED".to_string(),
            message: format!(
                "Starting protocol-aware scan of {} ports against {}",
                target_ports.len(),
                target_ip
            ),
            metadata: None,
        });

        let host_for_probes = host.clone();
        let ip_for_probes = target_ip.clone();

        let timeout_ms = profile.timeout_ms;
        let max_concurrent = profile.max_concurrent_ports;

        let probe_results = stream::iter(target_ports.iter().copied().map(|port| {
            let host = host_for_probes.clone();
            let ip = ip_for_probes.clone();
            async move { fallback_router::probe_with_fallback(&host, &ip, port, timeout_ms).await }
        }))
        .buffer_unordered(max_concurrent)
        .collect::<Vec<(PortProbeResult, Vec<ActivityEvent>)>>()
        .await;

        // Collect results and merge timelines
        let mut port_results: Vec<PortProbeResult> = Vec::new();
        for (result, probe_timeline) in probe_results {
            // Only add timeline events for open ports or interesting ones
            if result.state == PortState::Open || !probe_timeline.is_empty() {
                timeline.extend(probe_timeline);
            }
            port_results.push(result);
        }

        port_results.sort_by_key(|p| p.port);

        // === 3.5 Nmap Verification Fixture ===
        // Simulated truth layer fixture for testing
        let mock_truth_ports = vec![80, 443];
        nmap_validator::validate_parity(&mut port_results, &mock_truth_ports);

        // === 4. Final Summary ===
        let open_count = port_results
            .iter()
            .filter(|p| p.state == PortState::Open)
            .count();
        let total_count = port_results.len();
        let error_count = errors.len();

        timeline.push(ActivityEvent {
            timestamp: Utc::now(),
            severity: ActivitySeverity::Info,
            event_type: "SCAN_COMPLETED".to_string(),
            message: format!(
                "Scan completed: {}/{} ports open, {} errors",
                open_count, total_count, error_count
            ),
            metadata: Some(serde_json::json!({
                "open_ports": open_count,
                "total_ports": total_count,
                "errors": error_count,
            })),
        });

        let overall_status = if errors.is_empty() {
            CollectorStatus::Completed
        } else if open_count > 0 {
            CollectorStatus::PartialFailure
        } else {
            CollectorStatus::Failed
        };

        let mut snapshot = CollectorSnapshot {
            target_input: target_input.clone(),
            resolved_target,
            timestamp: Utc::now(),
            port_results,
            activity_timeline: timeline,
            errors,
            overall_status,
            diff: None,
        };

        // === 5. Change Detection (Diff Engine) ===
        if let Some(prev) = history_store::get_previous_snapshot(&target_input.normalized_url) {
            let diff = diff_engine::compare(&prev, &snapshot);

            snapshot.activity_timeline.push(ActivityEvent {
                timestamp: Utc::now(),
                severity: ActivitySeverity::Info,
                event_type: "DIFF_COMPLETED".to_string(),
                message: format!(
                    "Change detection completed against snapshot from {}: {}",
                    prev.timestamp.format("%H:%M:%S"),
                    diff.summaries.join(", ")
                ),
                metadata: Some(
                    serde_json::to_value(&diff.summaries).unwrap_or(serde_json::Value::Null),
                ),
            });

            snapshot.diff = Some(diff);
        } else {
            snapshot.activity_timeline.push(ActivityEvent {
                timestamp: Utc::now(),
                severity: ActivitySeverity::Info,
                event_type: "FIRST_SCAN".to_string(),
                message: "No previous history found for this target. Baseline established."
                    .to_string(),
                metadata: None,
            });
        }

        // Save current snapshot
        history_store::save_snapshot(&target_input.normalized_url, snapshot.clone());

        Ok(snapshot)
    }
}
