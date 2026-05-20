use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::domain::active_vuln::*;
use crate::domain::repositories::{ActiveScanRepository, ActiveFindingRepository};
use crate::infrastructure::active_detection::detectors::VulnDetector;

pub struct PerformActiveScan<'a, S: ActiveScanRepository, F: ActiveFindingRepository> {
    pub scan_repo: &'a S,
    pub finding_repo: &'a F,
    pub detectors: Arc<Vec<Box<dyn VulnDetector>>>,
    pub payload_selector: Arc<crate::infrastructure::active_detection::payload_selector::PayloadSelector>,
}

impl<'a, S: ActiveScanRepository, F: ActiveFindingRepository> PerformActiveScan<'a, S, F> {
    pub async fn execute(&self, config: ActiveScanConfig) -> Result<Uuid, String> {
        let scan_id = Uuid::new_v4();
        
        let initial_scan = ActiveScanResult {
            scan_id,
            target_url: config.target_url.clone(),
            status: ActiveScanStatus::Pending,
            start_time: Utc::now(),
            end_time: None,
            total_requests: 0,
            findings: vec![],
            errors: vec![],
            summary: ScanSummary::default(),
        };

        self.scan_repo.create_scan(initial_scan.clone()).await?;
        self.scan_repo.update_status(scan_id, ActiveScanStatus::Running).await?;

        let test_parameters = config.custom_parameters.unwrap_or_else(|| {
            vec!["id".into(), "page".into(), "q".into(), "url".into(), "file".into(), "token".into()]
        });

        // Safety Framework setup
        let rate_limit_ms = if config.rate_limit_rps > 0 {
            1000 / config.rate_limit_rps as u64
        } else {
            0
        };
        let waf_monitor = Arc::new(crate::infrastructure::safety::waf_monitor::WafMonitor::new());

        let mut all_findings = Vec::new();
        let mut total_requests = 0;
        let mut scan_errors = Vec::new();

        use futures::stream::{self, StreamExt};
        let client = reqwest::Client::new();
        let mut tasks = Vec::new();

        for param in &test_parameters {
            // Baseline Reflection Check
            let canary = format!("limma_canary_{}", Uuid::new_v4().to_string().replace("-", "")[..8].to_string());
            let is_reflected = if let Ok(resp) = client.get(&config.target_url).query(&[(param, &canary)]).send().await {
                if let Ok(body) = resp.text().await {
                    body.contains(&canary)
                } else { false }
            } else { false };

            // Differential Analysis Baseline
            let mut baseline = None;
            if let Ok(profile) = crate::infrastructure::active_detection::differential::build_baseline(&client, &config.target_url, param, "limma_safe_base").await {
                baseline = Some(profile);
            }

            // Context-Aware Heuristics
            let mut prioritized_types = crate::infrastructure::active_detection::heuristics::prioritize_vuln_types(param);
            
            // Filter noise: If no reflection, skip XSS and SSTI
            if !is_reflected {
                prioritized_types.retain(|t| *t != ActiveVulnType::ReflectedXss && *t != ActiveVulnType::ServerSideTemplateInjection);
            }

            // Create concurrent tasks for prioritized detectors
            for detector in self.detectors.iter() {
                let supported = detector.supported_types();
                
                // Intersection: Must be supported by detector, requested by user, AND prioritized by heuristics
                let should_run: Vec<_> = supported.into_iter()
                    .filter(|t| config.vuln_types.contains(t) && prioritized_types.contains(t))
                    .collect();

                if should_run.is_empty() {
                    continue; // Skip this detector for this parameter context
                }

                let detector_ref = detector.as_ref();
                let target_url = config.target_url.clone();
                let param_clone = param.clone();
                let wm = waf_monitor.clone();
                let baseline_clone = baseline.clone();
                let payload_selector_clone = self.payload_selector.clone();
                
                tasks.push(async move {
                    if wm.is_circuit_open(&target_url) {
                        return Err("Circuit Breaker Open: WAF blocked too many requests. Aborting scan.".to_string());
                    }

                    detector_ref.detect(&target_url, &param_clone, scan_id, payload_selector_clone.as_ref(), rate_limit_ms, wm, baseline_clone.as_ref())
                        .await
                        .map_err(|e| format!("Detector error for parameter {}: {}", param_clone, e))
                });
            }
        }

        // Execute Pipeline Concurrently
        let mut stream = stream::iter(tasks).buffer_unordered(5); // 5 concurrent detector-parameter chains
        while let Some(result) = stream.next().await {
            match result {
                Ok(findings) => {
                    total_requests += 1;
                    for f in findings {
                        if let Err(e) = self.finding_repo.save_finding(f.clone()).await {
                            scan_errors.push(format!("Failed to save finding: {}", e));
                        } else {
                            all_findings.push(f);
                        }
                    }
                }
                Err(e) => {
                    scan_errors.push(e);
                }
            }
            if waf_monitor.is_circuit_open(&config.target_url) {
                break;
            }
        }

        // Compute summary
        let mut summary = ScanSummary::default();
        for finding in &all_findings {
            let type_str = serde_json::to_string(&finding.vuln_type).unwrap_or("unknown".into());
            *summary.vuln_type_breakdown.entry(type_str).or_insert(0) += 1;
            
            use crate::domain::entities::SeverityLevel;
            match finding.severity {
                SeverityLevel::Critical => summary.critical_count += 1,
                SeverityLevel::High => summary.high_count += 1,
                SeverityLevel::Medium => summary.medium_count += 1,
                SeverityLevel::Low => summary.low_count += 1,
                SeverityLevel::Informational => summary.info_count += 1,
            }
        }
        
        summary.waf_detected = waf_monitor.is_waf_detected(&config.target_url);
        summary.waf_blocked_requests = waf_monitor.get_blocked_count(&config.target_url);

        // Finalize scan
        let _final_scan = ActiveScanResult {
            scan_id,
            target_url: config.target_url.clone(),
            status: ActiveScanStatus::Completed,
            start_time: initial_scan.start_time,
            end_time: Some(Utc::now()),
            total_requests,
            findings: all_findings,
            errors: scan_errors,
            summary: summary.clone(), // we need this if we update full result
        };

        // For simplicity, we just rely on updating status and stats could be an explicit update_scan.
        // If your repo needs a full update: (here we just mark completed for now)
        self.scan_repo.update_status(scan_id, ActiveScanStatus::Completed).await?;
        // Optional: Update scan with summary and end_time (needs repo method update, skipping for brevity)

        Ok(scan_id)
    }
}

