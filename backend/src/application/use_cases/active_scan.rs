use crate::domain::active_vuln::*;
use crate::domain::repositories::{ActiveFindingRepository, ActiveScanRepository};
use crate::infrastructure::active_detection::detectors::VulnDetector;
use crate::infrastructure::scan_controller::ScanTaskHandle;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use crate::domain::fuzzing::{EndpointContext, ScanTarget, InsertionPoint};
use crate::infrastructure::scanner::browser_crawler::BrowserCrawler;
use reqwest::Client;
use std::sync::atomic::Ordering;
use tokio::time::{sleep, Duration};

pub struct PerformActiveScan<S: ActiveScanRepository + Send + Sync, F: ActiveFindingRepository + Send + Sync> {
    pub scan_repo: Arc<S>,
    pub finding_repo: Arc<F>,
    pub detectors: Arc<Vec<Box<dyn VulnDetector>>>,
    pub payload_selector: Arc<crate::infrastructure::active_detection::payload_selector::PayloadSelector>,
}

impl<S: ActiveScanRepository + Send + Sync, F: ActiveFindingRepository + Send + Sync> PerformActiveScan<S, F> {
    pub async fn execute(&self, scan_id: Uuid, config: ActiveScanConfig, handle: ScanTaskHandle) -> Result<(), String> {
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

        let rate_limit_ms = if let Some(limit) = config.max_requests_per_endpoint {
            if limit > 0 { 1000 / limit as u64 } else { 0 }
        } else {
            0
        };
        let waf_monitor = Arc::new(crate::infrastructure::safety::waf_monitor::WafMonitor::new());

        let mut all_findings = Vec::new();
        let mut total_requests = 0;
        let mut scan_errors = Vec::new();

        use futures::stream::{self, StreamExt};
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
        let mut headers = HeaderMap::new();

        if let Some(token) = &config.bearer_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }
        if let Some(cookie) = &config.cookie {
            if let Ok(val) = HeaderValue::from_str(cookie) {
                headers.insert(reqwest::header::COOKIE, val);
            }
        }
        if let Some(custom) = &config.custom_headers {
            for line in custom.lines() {
                if let Some((k, v)) = line.split_once(':') {
                    if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(k.trim().as_bytes()), HeaderValue::from_str(v.trim())) {
                        headers.insert(name, val);
                    }
                }
            }
        }
        if let (Some(u), Some(p)) = (&config.basic_auth_user, &config.basic_auth_pass) {
            use base64::Engine;
            let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", u, p));
            if let Ok(val) = HeaderValue::from_str(&format!("Basic {}", auth)) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(if config.follow_redirects { reqwest::redirect::Policy::limited(5) } else { reqwest::redirect::Policy::none() })
            .build()
            .unwrap_or_default();

        let scan_specific_detectors = crate::infrastructure::active_detection::detectors::build_detectors(client.clone());
        let scan_detectors_arc = std::sync::Arc::new(scan_specific_detectors);

        let mut total_endpoints = 0;
        let mut total_parameters = 0;
        let mut api_routes_mapped = 0;
        let mut input_vectors_analyzed = 0;
        let mut auth_bounds_identified = 0;

        let mut tasks: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<ActiveVulnFinding>, String>> + Send>>> = Vec::new();

        if config.scan_mode == "fast" {
            let test_parameters = config.custom_parameters.unwrap_or_else(|| {
                vec!["id".into(), "page".into(), "q".into(), "url".into(), "file".into(), "token".into()]
            });

            total_endpoints = 1;
            total_parameters = test_parameters.len() as u32;
            input_vectors_analyzed = total_parameters;

            for param in test_parameters {
                let canary_id = Uuid::new_v4().to_string().replace("-", "")[..8].to_string();
                let canary = format!("limma_canary_{}", canary_id);
                let is_reflected = if let Ok(resp) = client.get(&config.target_url).query(&[(&param, &canary)]).send().await {
                    if let Ok(body) = resp.text().await { body.contains(&canary) } else { false }
                } else { false };

                let mut baseline = None;
                if let Ok(profile) = crate::infrastructure::active_detection::differential::build_baseline(
                    &client, &config.target_url, &param, "limma_safe_base"
                ).await { baseline = Some(profile); }

                let mut prioritized_types = crate::infrastructure::active_detection::heuristics::prioritize_vuln_types(&param);
                if !is_reflected {
                    prioritized_types.retain(|t| *t != ActiveVulnType::ReflectedXss && *t != ActiveVulnType::ServerSideTemplateInjection);
                }

                let endpoint_ctx = EndpointContext::new("GET", &config.target_url);
                let insertion_point = InsertionPoint::QueryParam(param.clone());

                for detector in scan_detectors_arc.iter() {
                    let supported = detector.supported_types();
                    let should_run: Vec<_> = supported.into_iter().filter(|t| config.vuln_types.contains(t) && prioritized_types.contains(t)).collect();
                    if should_run.is_empty() { continue; }

                    let detector_ref = detector.as_ref();
                    let target_url = config.target_url.clone();
                    let param_clone = param.clone();
                    let wm = waf_monitor.clone();
                    let baseline_clone = baseline.clone();
                    let payload_selector_clone = self.payload_selector.clone();
                    let endpoint_ctx_clone = endpoint_ctx.clone();
                    let insertion_point_clone = insertion_point.clone();

                    tasks.push(Box::pin(async move {
                        if wm.is_circuit_open(&target_url) {
                            return Err("Circuit Breaker Open: WAF blocked too many requests. Aborting scan.".to_string());
                        }
                        detector_ref.detect(
                            &target_url,
                            &param_clone,
                            scan_id,
                            payload_selector_clone.as_ref(),
                            rate_limit_ms,
                            wm,
                            baseline_clone.as_ref(),
                            Some(&endpoint_ctx_clone),
                            Some(&insertion_point_clone),
                        ).await.map_err(|e| format!("Detector error for parameter {}: {}", param_clone, e))
                    }));
                }
            }
        } else {
            // MODERN SPA / DEEP API SCAN
            let mut scan_targets = Vec::new();
            if config.enable_headless_browser {
                let max_tabs = Some(config.max_browser_tabs as usize);
                let crawler = BrowserCrawler::new(&config.target_url, max_tabs);
                match crawler.crawl() {
                    Ok(discovered_result) => scan_targets.extend(discovered_result.endpoints),
                    Err(e) => scan_errors.push(format!("Crawler failed: {}", e)),
                }
            } else {
                let base_ctx = EndpointContext::new("GET", &config.target_url);
                scan_targets.push(ScanTarget {
                    endpoint: base_ctx,
                    insertion_points: vec![InsertionPoint::QueryParam("id".into())],
                });
            }

            total_endpoints = scan_targets.len() as u32;
            let mut unique_parameters = std::collections::HashSet::new();

            let has_auth = config.bearer_token.is_some() || config.cookie.is_some() || config.basic_auth_user.is_some() || config.custom_headers.is_some();
            let unauth_client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap_or_default();

            for target in &scan_targets {
                input_vectors_analyzed += target.insertion_points.len() as u32;
                for ip in &target.insertion_points {
                    match ip {
                        InsertionPoint::QueryParam(p) => { unique_parameters.insert(p.clone()); },
                        InsertionPoint::FormData(p) => { unique_parameters.insert(p.clone()); },
                        InsertionPoint::JsonBodyPath(p) => { unique_parameters.insert(p.clone()); },
                        InsertionPoint::Header(h) => { unique_parameters.insert(h.clone()); },
                    }
                }

                if target.endpoint.url.contains("/api/") || target.endpoint.url.ends_with(".json") {
                    api_routes_mapped += 1;
                }

                if has_auth && target.endpoint.method == "GET" {
                    if let Ok(u_resp) = unauth_client.get(&target.endpoint.url).send().await {
                        let u_status = u_resp.status().as_u16();
                        if u_status == 401 || u_status == 403 || u_status == 302 {
                            if let Ok(a_resp) = client.get(&target.endpoint.url).send().await {
                                let a_status = a_resp.status().as_u16();
                                if a_status == 200 || a_status == 201 {
                                    auth_bounds_identified += 1;
                                }
                            }
                        }
                    }
                }
            }
            total_parameters = unique_parameters.len() as u32;

            for target in scan_targets {
                if !config.allow_destructive_methods && ["PUT", "PATCH", "DELETE"].contains(&target.endpoint.method.to_uppercase().as_str()) {
                    continue;
                }

                for insertion_point in target.insertion_points {
                    for detector in self.detectors.iter() {
                        let supported = detector.supported_types();
                        let should_run: Vec<_> = supported.into_iter().filter(|t| config.vuln_types.contains(t)).collect();
                        if should_run.is_empty() { continue; }

                        let detector_ref = detector.as_ref();
                        let target_url = config.target_url.clone();
                        let endpoint_ctx_clone = target.endpoint.clone();
                        let insertion_point_clone = insertion_point.clone();
                        let wm = waf_monitor.clone();
                        let payload_selector_clone = self.payload_selector.clone();
                        
                        let param_fallback = match &insertion_point_clone {
                            InsertionPoint::QueryParam(p) => p.clone(),
                            InsertionPoint::FormData(p) => p.clone(),
                            InsertionPoint::JsonBodyPath(p) => p.clone(),
                            InsertionPoint::Header(h) => h.clone(),
                        };

                        tasks.push(Box::pin(async move {
                            if wm.is_circuit_open(&target_url) {
                                return Err("Circuit Breaker Open: WAF blocked too many requests. Aborting scan.".to_string());
                            }
                            detector_ref.detect(
                                &target_url,
                                &param_fallback, // parameter fallback
                                scan_id,
                                payload_selector_clone.as_ref(),
                                rate_limit_ms,
                                wm,
                                None, // no baseline for dynamic APIs yet
                                Some(&endpoint_ctx_clone),
                                Some(&insertion_point_clone),
                            ).await.map_err(|e| format!("Detector error: {}", e))
                        }));
                    }
                }
            }
        }

        let mut stream = stream::iter(tasks).buffer_unordered(5);
        while let Some(result) = stream.next().await {
            // Check for cancel
            if handle.is_cancelled.load(Ordering::SeqCst) {
                tracing::info!("Scan {} cancelled by user", scan_id);
                self.scan_repo.update_status(scan_id, ActiveScanStatus::Cancelled).await?;
                return Ok(());
            }

            // Check for pause
            while handle.is_paused.load(Ordering::SeqCst) {
                if handle.is_cancelled.load(Ordering::SeqCst) {
                    tracing::info!("Scan {} cancelled while paused", scan_id);
                    self.scan_repo.update_status(scan_id, ActiveScanStatus::Cancelled).await?;
                    return Ok(());
                }
                sleep(Duration::from_millis(500)).await;
            }

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
                Err(e) => scan_errors.push(e),
            }
            if waf_monitor.is_circuit_open(&config.target_url) { break; }
        }

        // Final check before marking complete
        if handle.is_cancelled.load(Ordering::SeqCst) {
            self.scan_repo.update_status(scan_id, ActiveScanStatus::Cancelled).await?;
            return Ok(());
        }

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
        
        summary.total_endpoints = total_endpoints;
        summary.total_parameters = total_parameters;
        summary.api_routes_mapped = api_routes_mapped;
        summary.input_vectors_analyzed = input_vectors_analyzed;
        summary.auth_bounds_identified = auth_bounds_identified;

        self.scan_repo.update_status(scan_id, ActiveScanStatus::Completed).await?;

        Ok(())
    }
}
