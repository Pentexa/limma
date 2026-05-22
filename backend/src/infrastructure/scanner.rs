pub mod consistency;
pub mod correlation;
pub mod crawler;
pub mod fingerprint;
pub mod security;

use crate::domain::entities::{CertaintyLevel, CertaintyNote, WebScanResult};
use crate::domain::repositories::WebsiteScanner;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{redirect::Policy, Client};
use std::collections::HashMap;
use std::time::Instant;

pub struct HttpWebsiteScanner {
    fingerprinter: fingerprint::FingerprintEngine,
    waf_monitor: crate::infrastructure::safety::waf_monitor::WafMonitor,
}

impl HttpWebsiteScanner {
    pub fn new() -> Self {
        Self {
            fingerprinter: fingerprint::FingerprintEngine::new(),
            waf_monitor: crate::infrastructure::safety::waf_monitor::WafMonitor::new(),
        }
    }

    fn build_client(&self, profile: &crate::domain::engine_config::EngineConfig) -> Client {
        let mut builder = Client::builder()
            .user_agent(&profile.user_agent)
            .timeout(std::time::Duration::from_millis(profile.timeout_ms))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .tcp_keepalive(std::time::Duration::from_secs(15));

        if !profile.follow_redirects {
            builder = builder.redirect(Policy::none());
        }

        if profile.use_proxy {
            if let Some(proxy_url) = &profile.proxy_url {
                if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }

        builder
            .build()
            .expect("Failed to build profile-specific HTTP client")
    }
}

#[async_trait]
impl WebsiteScanner for HttpWebsiteScanner {
    async fn scan(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<WebScanResult, String> {
        let scan_start_time = Utc::now();
        let total_start = Instant::now();

        let client = self.build_client(profile);
        let mut crawl_res =
            crawler::crawl(&client, &self.fingerprinter, url, profile, None).await?;
        let summary =
            consistency::analyze_consistency(&mut crawl_res.pages, &mut crawl_res.events, &None);
        let correlation_report =
            correlation::analyze_target(&crawl_res.pages, &mut crawl_res.events, &None);

        let scan_end_time = Utc::now();
        let total_duration_ms = total_start.elapsed().as_millis() as u64;

        let main_page = crawl_res.pages.first().cloned();

        let mut final_status_code = 0;
        let mut latency_ms = 0;
        let mut headers = HashMap::new();
        let mut content_type = None;
        let mut content_length = None;
        let mut server = None;
        let mut cache_control = None;
        let mut detected_technologies = Vec::new();
        let mut security_headers = Vec::new();
        let mut risk_insights = Vec::new();
        let mut security_score = 0;
        let mut has_page_data = false;

        if let Some(mp) = main_page {
            final_status_code = mp.status_code;
            latency_ms = mp.latency_ms;
            headers = mp.headers.clone();
            content_type = mp.content_type.clone();

            for (k, v) in &mp.headers {
                match k.as_str() {
                    "content-length" => content_length = v.parse::<u64>().ok(),
                    "server" => server = Some(v.clone()),
                    "cache-control" => cache_control = Some(v.clone()),
                    _ => {}
                }
            }

            detected_technologies = mp.detected_technologies;
            security_headers = mp.security_headers.clone();
            risk_insights = mp.risk_insights.clone();
            let waf = self.waf_monitor.fingerprint_waf(url, &mp.headers, "");
            if let Some(w) = waf {
                risk_insights.push(crate::domain::entities::RiskInsight {
                    title: format!("WAF Detected: {}", w),
                    severity: crate::domain::entities::RiskSeverity::Medium,
                    explanation: "WAF may block some scanning actions.".to_string(),
                    evidence: w.clone(),
                });
            }
            
            security_score = security::calculate_security_score(&security_headers, &risk_insights);
            has_page_data = true;
        }

        let scan_certainty = if has_page_data && (200..400).contains(&final_status_code) {
            Some(CertaintyNote {
                level: CertaintyLevel::Certain,
                reason: "Sayfa başarıyla tarandı ve analiz edildi".to_string(),
            })
        } else if has_page_data {
            Some(CertaintyNote {
                level: CertaintyLevel::Likely,
                reason: format!(
                    "Sayfa tarandı ama HTTP {} yanıtı alındı — bazı veriler eksik olabilir",
                    final_status_code
                ),
            })
        } else {
            Some(CertaintyNote {
                level: CertaintyLevel::Unknown,
                reason: "Hiçbir sayfa taranamadı — sonuçlar güvenilir değil".to_string(),
            })
        };

        Ok(WebScanResult {
            original_target_url: url.to_string(),
            final_url: crawl_res.final_url,
            scan_start_time,
            scan_end_time,
            total_duration_ms,
            final_status_code,
            latency_ms,
            redirect_count: crawl_res.main_chain.len().saturating_sub(1) as u32,
            redirect_chain: crawl_res.main_chain,
            headers,
            content_type,
            content_length,
            server,
            cache_control,
            detected_technologies,
            security_headers,
            risk_insights,
            security_score,
            pages: crawl_res.pages,
            timeline: crawl_res.events,
            summary: Some(summary),
            correlation: Some(correlation_report),
            scan_certainty,
        })
    }

    async fn scan_stream(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::ScanEvent>,
    ) -> Result<WebScanResult, String> {
        let scan_start_time = Utc::now();
        let total_start = Instant::now();

        let client = self.build_client(profile);
        let mut crawl_res =
            crawler::crawl(&client, &self.fingerprinter, url, profile, Some(tx.clone())).await?;
        let summary = consistency::analyze_consistency(
            &mut crawl_res.pages,
            &mut crawl_res.events,
            &Some(tx.clone()),
        );
        let correlation_report =
            correlation::analyze_target(&crawl_res.pages, &mut crawl_res.events, &Some(tx.clone()));

        let scan_end_time = Utc::now();
        let total_duration_ms = total_start.elapsed().as_millis() as u64;

        let main_page = crawl_res.pages.first().cloned();

        let mut final_status_code = 0;
        let mut latency_ms = 0;
        let mut headers = HashMap::new();
        let mut content_type = None;
        let mut content_length = None;
        let mut server = None;
        let mut cache_control = None;
        let mut detected_technologies = Vec::new();
        let mut security_headers = Vec::new();
        let mut risk_insights = Vec::new();
        let mut security_score = 0;
        let mut has_page_data = false;

        if let Some(mp) = main_page {
            final_status_code = mp.status_code;
            latency_ms = mp.latency_ms;
            headers = mp.headers.clone();
            content_type = mp.content_type.clone();

            for (k, v) in &mp.headers {
                match k.as_str() {
                    "content-length" => content_length = v.parse::<u64>().ok(),
                    "server" => server = Some(v.clone()),
                    "cache-control" => cache_control = Some(v.clone()),
                    _ => {}
                }
            }

            detected_technologies = mp.detected_technologies;
            security_headers = mp.security_headers.clone();
            risk_insights = mp.risk_insights.clone();
            let waf = self.waf_monitor.fingerprint_waf(url, &mp.headers, "");
            if let Some(w) = waf {
                risk_insights.push(crate::domain::entities::RiskInsight {
                    title: format!("WAF Detected: {}", w),
                    severity: crate::domain::entities::RiskSeverity::Medium,
                    explanation: "WAF may block some scanning actions.".to_string(),
                    evidence: w.clone(),
                });
            }
            
            security_score = security::calculate_security_score(&security_headers, &risk_insights);
            has_page_data = true;
        }

        let scan_certainty = if has_page_data && (200..400).contains(&final_status_code) {
            Some(CertaintyNote {
                level: CertaintyLevel::Certain,
                reason: "Sayfa başarıyla tarandı ve analiz edildi".to_string(),
            })
        } else if has_page_data {
            Some(CertaintyNote {
                level: CertaintyLevel::Likely,
                reason: format!(
                    "Sayfa tarandı ama HTTP {} yanıtı alındı — bazı veriler eksik olabilir",
                    final_status_code
                ),
            })
        } else {
            Some(CertaintyNote {
                level: CertaintyLevel::Unknown,
                reason: "Hiçbir sayfa taranamadı — sonuçlar güvenilir değil".to_string(),
            })
        };

        let result = WebScanResult {
            original_target_url: url.to_string(),
            final_url: crawl_res.final_url,
            scan_start_time,
            scan_end_time,
            total_duration_ms,
            final_status_code,
            latency_ms,
            redirect_count: crawl_res.main_chain.len().saturating_sub(1) as u32,
            redirect_chain: crawl_res.main_chain,
            headers,
            content_type,
            content_length,
            server,
            cache_control,
            detected_technologies,
            security_headers,
            risk_insights,
            security_score,
            pages: crawl_res.pages,
            timeline: crawl_res.events,
            summary: Some(summary),
            correlation: Some(correlation_report),
            scan_certainty,
        };

        let _ = tx.send(crate::domain::entities::ScanEvent {
            timestamp: Utc::now(),
            event_type: "FINAL_RESULT".to_string(),
            level: "INFO".to_string(),
            message: "Scan successfully completed".to_string(),
            payload: Some(serde_json::to_value(&result).unwrap_or(serde_json::Value::Null)),
        });

        Ok(result)
    }
}
