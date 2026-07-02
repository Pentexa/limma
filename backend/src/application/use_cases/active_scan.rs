use crate::domain::active_vuln::*;
use crate::domain::fuzzing::{EndpointContext, InsertionPoint, ScanTarget};
use crate::domain::repositories::{ActiveFindingRepository, ActiveScanRepository};
use crate::infrastructure::active_detection::detectors::VulnDetector;
use crate::infrastructure::scan_controller::ScanTaskHandle;
use crate::infrastructure::scanner::browser_crawler::BrowserCrawler;
use chrono::Utc;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::time::{sleep, timeout_at, Duration, Instant};
use uuid::Uuid;

type DetectorTask<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<ActiveVulnFinding>, String>> + Send + 'a>>;

#[derive(Clone, Copy)]
struct ScanMetrics {
    total_endpoints: u32,
    total_parameters: u32,
    api_routes_mapped: u32,
    input_vectors_analyzed: u32,
    auth_bounds_identified: u32,
}

pub struct PerformActiveScan<
    S: ActiveScanRepository + Send + Sync,
    F: ActiveFindingRepository + Send + Sync,
> {
    pub scan_repo: Arc<S>,
    pub finding_repo: Arc<F>,
    pub detectors: Arc<Vec<Box<dyn VulnDetector>>>,
    pub payload_selector:
        Arc<crate::infrastructure::active_detection::payload_selector::PayloadSelector>,
}

impl<S: ActiveScanRepository + Send + Sync, F: ActiveFindingRepository + Send + Sync>
    PerformActiveScan<S, F>
{
    async fn persist_scan_result(&self, result: ActiveScanResult) -> Result<(), String> {
        self.scan_repo.update_scan(result).await
    }

    pub async fn execute(
        &self,
        scan_id: Uuid,
        config: ActiveScanConfig,
        handle: ScanTaskHandle,
    ) -> Result<(), String> {
        let start_time = Utc::now();
        let initial_scan = ActiveScanResult {
            scan_id,
            target_url: config.target_url.clone(),
            status: ActiveScanStatus::Pending,
            start_time,
            end_time: None,
            total_requests: 0,
            findings: vec![],
            errors: vec![],
            summary: ScanSummary::default(),
        };

        self.scan_repo.create_scan(initial_scan.clone()).await?;
        self.scan_repo
            .update_status(scan_id, ActiveScanStatus::Running)
            .await?;

        let rate_limit_ms = if let Some(limit) = config.max_requests_per_endpoint {
            if limit > 0 {
                1000 / limit as u64
            } else {
                0
            }
        } else {
            0
        };
        let waf_monitor = Arc::new(crate::infrastructure::safety::waf_monitor::WafMonitor::new());
        let scan_duration_limit_sec = config.max_scan_duration_sec.filter(|seconds| *seconds > 0);
        let scan_deadline = scan_duration_limit_sec
            .map(|seconds| Instant::now() + Duration::from_secs(seconds as u64));

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
                    if let (Ok(name), Ok(val)) = (
                        HeaderName::from_bytes(k.trim().as_bytes()),
                        HeaderValue::from_str(v.trim()),
                    ) {
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
            .timeout(Duration::from_secs(15))
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(5)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
            .unwrap_or_default();

        let scan_specific_detectors =
            crate::infrastructure::active_detection::detectors::build_detectors(client.clone());
        let scan_detectors_arc = std::sync::Arc::new(scan_specific_detectors);

        let total_endpoints: u32;
        let total_parameters: u32;
        let mut api_routes_mapped: u32;
        let mut input_vectors_analyzed: u32;
        let mut auth_bounds_identified: u32;

        let mut tasks: Vec<DetectorTask<'_>> = Vec::new();

        if config.scan_mode == "fast" {
            let test_parameters = config.custom_parameters.unwrap_or_else(|| {
                vec![
                    "id".into(),
                    "page".into(),
                    "q".into(),
                    "url".into(),
                    "file".into(),
                    "token".into(),
                ]
            });

            total_endpoints = 1;
            total_parameters = test_parameters.len() as u32;
            api_routes_mapped = 0;
            input_vectors_analyzed = total_parameters;
            auth_bounds_identified = 0;

            for param in test_parameters {
                let canary_id = Uuid::new_v4().to_string().replace("-", "")[..8].to_string();
                let canary = format!("limma_canary_{}", canary_id);
                let is_reflected = if let Ok(resp) = client
                    .get(&config.target_url)
                    .query(&[(&param, &canary)])
                    .send()
                    .await
                {
                    if let Ok(body) = resp.text().await {
                        body.contains(&canary)
                    } else {
                        false
                    }
                } else {
                    false
                };

                let mut baseline = None;
                if let Ok(profile) =
                    crate::infrastructure::active_detection::differential::build_baseline(
                        &client,
                        &config.target_url,
                        &param,
                        "limma_safe_base",
                    )
                    .await
                {
                    baseline = Some(profile);
                }

                let mut prioritized_types =
                    crate::infrastructure::active_detection::heuristics::prioritize_vuln_types(
                        &param,
                    );
                if !is_reflected {
                    prioritized_types.retain(|t| {
                        *t != ActiveVulnType::ReflectedXss
                            && *t != ActiveVulnType::ServerSideTemplateInjection
                    });
                }

                let endpoint_ctx = EndpointContext::new("GET", &config.target_url);
                let insertion_point = InsertionPoint::QueryParam(param.clone());

                for detector in scan_detectors_arc.iter() {
                    let supported = detector.supported_types();
                    let should_run: Vec<_> = supported
                        .into_iter()
                        .filter(|t| config.vuln_types.contains(t) && prioritized_types.contains(t))
                        .collect();
                    if should_run.is_empty() {
                        continue;
                    }

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
            api_routes_mapped = 0;
            input_vectors_analyzed = 0;
            auth_bounds_identified = 0;
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

            let has_auth = config.bearer_token.is_some()
                || config.cookie.is_some()
                || config.basic_auth_user.is_some()
                || config.custom_headers.is_some();
            let unauth_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap_or_default();

            for target in &scan_targets {
                input_vectors_analyzed += target.insertion_points.len() as u32;
                for ip in &target.insertion_points {
                    match ip {
                        InsertionPoint::QueryParam(p) => {
                            unique_parameters.insert(p.clone());
                        }
                        InsertionPoint::FormData(p) => {
                            unique_parameters.insert(p.clone());
                        }
                        InsertionPoint::JsonBodyPath(p) => {
                            unique_parameters.insert(p.clone());
                        }
                        InsertionPoint::Header(h) => {
                            unique_parameters.insert(h.clone());
                        }
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

            let mut skipped_destructive_targets = 0u32;
            for target in scan_targets {
                let method_upper = target.endpoint.method.to_uppercase();
                let is_destructive = ["PUT", "PATCH", "DELETE"].contains(&method_upper.as_str());
                if is_destructive
                    && (!config.allow_destructive_methods
                        || !config.l3_consent_accepted
                        || config.safe_mode)
                {
                    skipped_destructive_targets += 1;
                    continue;
                }

                for insertion_point in target.insertion_points {
                    if matches!(insertion_point, InsertionPoint::JsonBodyPath(_))
                        && !config.enable_json_fuzzing
                    {
                        continue;
                    }

                    let baseline =
                        match crate::infrastructure::active_detection::differential::build_baseline_for_insertion(
                            &client,
                            &config.target_url,
                            &target.endpoint,
                            &insertion_point,
                            "limma_safe_base",
                        )
                        .await
                        {
                            Ok(profile) => Some(profile),
                            Err(e) => {
                                tracing::debug!(
                                    "Deep scan baseline failed for {} {:?}: {}",
                                    target.endpoint.url,
                                    insertion_point,
                                    e
                                );
                                None
                            }
                        };

                    for detector in scan_detectors_arc.iter() {
                        let supported = detector.supported_types();
                        let should_run: Vec<_> = supported
                            .into_iter()
                            .filter(|t| config.vuln_types.contains(t))
                            .collect();
                        if should_run.is_empty() {
                            continue;
                        }

                        let detector_ref = detector.as_ref();
                        let target_url = config.target_url.clone();
                        let endpoint_ctx_clone = target.endpoint.clone();
                        let insertion_point_clone = insertion_point.clone();
                        let wm = waf_monitor.clone();
                        let payload_selector_clone = self.payload_selector.clone();
                        let baseline_clone = baseline.clone();

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
                                baseline_clone.as_ref(),
                                Some(&endpoint_ctx_clone),
                                Some(&insertion_point_clone),
                            ).await.map_err(|e| format!("Detector error: {}", e))
                        }));
                    }
                }
            }

            if skipped_destructive_targets > 0 {
                scan_errors.push(format!(
                    "Skipped {} destructive endpoint(s): require safe_mode=false, allow_destructive_methods=true, and l3_consent_accepted=true",
                    skipped_destructive_targets
                ));
            }
        }

        if tasks.is_empty() {
            scan_errors
                .push("No detector tasks scheduled for this active scan configuration".to_string());
        }

        let metrics = ScanMetrics {
            total_endpoints,
            total_parameters,
            api_routes_mapped,
            input_vectors_analyzed,
            auth_bounds_identified,
        };

        let mut stream = stream::iter(tasks).buffer_unordered(5);
        loop {
            let result = match scan_deadline {
                Some(deadline) => match timeout_at(deadline, stream.next()).await {
                    Ok(result) => result,
                    Err(_) => {
                        scan_errors.push(format!(
                            "Scan duration limit exceeded after {} seconds",
                            scan_duration_limit_sec.unwrap_or_default()
                        ));
                        break;
                    }
                },
                None => stream.next().await,
            };

            let Some(result) = result else {
                break;
            };

            // Check for cancel
            if handle.is_cancelled.load(Ordering::SeqCst) {
                tracing::info!("Scan {} cancelled by user", scan_id);
                scan_errors.push("Scan cancelled by user".to_string());
                let summary =
                    build_scan_summary(&all_findings, &waf_monitor, &config.target_url, metrics);
                self.persist_scan_result(ActiveScanResult {
                    scan_id,
                    target_url: config.target_url.clone(),
                    status: ActiveScanStatus::Cancelled,
                    start_time,
                    end_time: Some(Utc::now()),
                    total_requests,
                    findings: all_findings,
                    errors: scan_errors,
                    summary,
                })
                .await?;
                return Ok(());
            }

            // Check for pause
            while handle.is_paused.load(Ordering::SeqCst) {
                if handle.is_cancelled.load(Ordering::SeqCst) {
                    tracing::info!("Scan {} cancelled while paused", scan_id);
                    scan_errors.push("Scan cancelled while paused".to_string());
                    let summary = build_scan_summary(
                        &all_findings,
                        &waf_monitor,
                        &config.target_url,
                        metrics,
                    );
                    self.persist_scan_result(ActiveScanResult {
                        scan_id,
                        target_url: config.target_url.clone(),
                        status: ActiveScanStatus::Cancelled,
                        start_time,
                        end_time: Some(Utc::now()),
                        total_requests,
                        findings: all_findings,
                        errors: scan_errors,
                        summary,
                    })
                    .await?;
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
            if waf_monitor.is_circuit_open(&config.target_url) {
                scan_errors.push("Circuit breaker opened after repeated WAF blocks".to_string());
                break;
            }
        }

        // Final check before marking complete
        if handle.is_cancelled.load(Ordering::SeqCst) {
            scan_errors.push("Scan cancelled by user".to_string());
            let summary =
                build_scan_summary(&all_findings, &waf_monitor, &config.target_url, metrics);
            self.persist_scan_result(ActiveScanResult {
                scan_id,
                target_url: config.target_url.clone(),
                status: ActiveScanStatus::Cancelled,
                start_time,
                end_time: Some(Utc::now()),
                total_requests,
                findings: all_findings,
                errors: scan_errors,
                summary,
            })
            .await?;
            return Ok(());
        }

        let summary = build_scan_summary(&all_findings, &waf_monitor, &config.target_url, metrics);

        let final_status = if total_requests == 0 && !scan_errors.is_empty() {
            ActiveScanStatus::Failed
        } else {
            ActiveScanStatus::Completed
        };

        self.persist_scan_result(ActiveScanResult {
            scan_id,
            target_url: config.target_url.clone(),
            status: final_status,
            start_time,
            end_time: Some(Utc::now()),
            total_requests,
            findings: all_findings,
            errors: scan_errors,
            summary,
        })
        .await?;

        Ok(())
    }
}

fn build_scan_summary(
    findings: &[ActiveVulnFinding],
    waf_monitor: &crate::infrastructure::safety::waf_monitor::WafMonitor,
    target_url: &str,
    metrics: ScanMetrics,
) -> ScanSummary {
    let mut summary = ScanSummary::default();
    for finding in findings {
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

    summary.waf_detected = waf_monitor.is_waf_detected(target_url);
    summary.waf_blocked_requests = waf_monitor.get_blocked_count(target_url);
    summary.total_endpoints = metrics.total_endpoints;
    summary.total_parameters = metrics.total_parameters;
    summary.api_routes_mapped = metrics.api_routes_mapped;
    summary.input_vectors_analyzed = metrics.input_vectors_analyzed;
    summary.auth_bounds_identified = metrics.auth_bounds_identified;
    summary
}
