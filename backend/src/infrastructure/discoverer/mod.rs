pub mod classifier;
pub mod fetcher;
pub mod html_analyzer;
pub mod js_collector;
pub mod js_static_analyzer;
pub mod normalizer;

use crate::domain::entities::{ApiDiscoveryResult, CertaintyLevel, CertaintyNote, EndpointDetail};
use crate::domain::repositories::ApiDiscoverer;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use url::Url;

use classifier::EndpointClassifier;
use fetcher::CrawlerFetcher;
use html_analyzer::HtmlAnalyzer;
use js_collector::JsCollector;
use js_static_analyzer::JsStaticAnalyzer;
use normalizer::PathNormalizer;

pub struct HttpApiDiscoverer {}

impl Default for HttpApiDiscoverer {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpApiDiscoverer {
    pub fn new() -> Self {
        Self {}
    }

    fn build_fetcher(
        &self,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> CrawlerFetcher {
        let mut builder = reqwest::ClientBuilder::new()
            .user_agent(&profile.user_agent)
            .timeout(std::time::Duration::from_millis(profile.timeout_ms))
            .cookie_store(true)
            .brotli(true)
            .gzip(true)
            .deflate(true);

        if profile.use_proxy {
            if let Some(proxy_url) = &profile.proxy_url {
                if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }

        let client = builder.build().unwrap_or_else(|_| reqwest::Client::new());
        CrawlerFetcher::new(client, profile.rate_limiter.clone())
    }

    #[allow(clippy::too_many_arguments)]
    fn register_path(
        &self,
        endpoints_map: &mut HashMap<String, EndpointDetail>,
        raw_path: &str,
        base_url: &Url,
        source_type: &str,
        snippet: &str,
        method: &str,
        params: Vec<String>,
        auth_prob: f32,
        reason: &str,
        line_number: Option<usize>,
    ) {
        if EndpointClassifier::is_false_positive(raw_path) {
            return;
        }

        let conf = EndpointClassifier::score_confidence(source_type, method);
        PathNormalizer::resolve_and_merge(
            endpoints_map,
            raw_path,
            base_url,
            source_type,
            snippet,
            method,
            params,
            auth_prob,
            conf,
            reason,
            line_number,
        );
    }
}

#[async_trait]
impl ApiDiscoverer for HttpApiDiscoverer {
    async fn discover(
        &self,
        url_str: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<ApiDiscoveryResult, String> {
        let base_url = Url::parse(url_str).map_err(|e| format!("Invalid Base URL: {}", e))?;

        let fetcher = self.build_fetcher(profile);

        // 1. Fetcher layer
        let html_body = fetcher.fetch_html(url_str).await?;

        let mut endpoints_map: HashMap<String, EndpointDetail> = HashMap::new();
        let mut tech_stack = HashSet::new();

        // 2. HTML Analyzer layer
        let html_res = HtmlAnalyzer::parse(&html_body, &base_url);

        let actual_base_url = if let Some(href) = html_res.base_href {
            Url::parse(&href).unwrap_or(base_url.clone())
        } else {
            base_url.clone()
        };

        for (action, method, params) in html_res.forms {
            let snippet = format!("<form action='{}' method='{}'>", action, method);
            self.register_path(
                &mut endpoints_map,
                &action,
                &actual_base_url,
                "HTML Form",
                &snippet,
                &method,
                params,
                0.10,
                "Explicit HTML Form element action pointing to an endpoint",
                None,
            );
        }

        for link in html_res.links {
            if link.contains("/api/") || link.contains("api.") {
                let snippet = format!("<a href='{}'>", link);
                self.register_path(
                    &mut endpoints_map,
                    &link,
                    &actual_base_url,
                    "HTML Link",
                    &snippet,
                    "GET",
                    vec![],
                    0.0,
                    "HTML Anchor link referencing an API path",
                    None,
                );
            }
        }

        for path in html_res.data_endpoints {
            let snippet = format!("Data/Meta Attribute: {}", path);
            self.register_path(
                &mut endpoints_map,
                &path,
                &actual_base_url,
                "Data Attribute / Meta",
                &snippet,
                "UNKNOWN",
                vec![],
                0.05,
                "Data or Meta attribute holding an endpoint configuration",
                None,
            );
        }

        let mut js_sources_to_eval = Vec::new();
        for inline in html_res.inline_scripts {
            js_sources_to_eval.push((inline, "Inline Script"));
        }

        // 3. JS Collector
        for src in html_res.external_js_src.into_iter().take(15) {
            if let Ok(js_url) = actual_base_url.join(&src) {
                let host = js_url.host_str().unwrap_or("");
                let orig_host = actual_base_url.host_str().unwrap_or("");
                if host == orig_host || host.contains("cdn") || host.contains("unpkg") {
                    if let Ok(js_code) = fetcher.fetch_js(js_url.as_str()).await {
                        let metadata = JsCollector::analyze_metadata(&js_code);

                        for chunk in metadata.potential_chunks.into_iter().take(5) {
                            if let Ok(chunk_url) = actual_base_url.join(&chunk) {
                                if let Ok(chunk_code) = fetcher.fetch_js(chunk_url.as_str()).await {
                                    js_sources_to_eval
                                        .push((chunk_code, "External JS (Dynamic Import)"));
                                }
                            }
                        }
                        js_sources_to_eval.push((js_code, "External JS (XHR/Fetch)"));
                    }
                }
            }
        }

        // 4. JS Static Analyzer evaluation
        for (code, src_label) in js_sources_to_eval {
            let analyzer_res = JsStaticAnalyzer::analyze(&code);
            for stack in analyzer_res.tech_stack {
                tech_stack.insert(stack.to_string());
            }

            for (path, snippet, reason, line_num) in analyzer_res.paths_with_evidence {
                let auth_prob = EndpointClassifier::assess_auth(&code, &path);

                let mut method = "UNKNOWN";

                // Heuristics based on snippet traces
                let trace_lower = snippet.to_lowercase();
                if trace_lower.contains(".post")
                    || trace_lower.contains("method: 'post'")
                    || trace_lower.contains("method:\"post\"")
                {
                    method = "POST";
                } else if trace_lower.contains(".put") || trace_lower.contains("method: 'put'") {
                    method = "PUT";
                } else if trace_lower.contains(".delete")
                    || trace_lower.contains("method: 'delete'")
                {
                    method = "DELETE";
                } else if trace_lower.contains(".get") || trace_lower.contains("method: 'get'") {
                    method = "GET";
                }

                let clean_path = path.replace("[VAR]", "");
                self.register_path(
                    &mut endpoints_map,
                    &clean_path,
                    &actual_base_url,
                    src_label,
                    &snippet,
                    method,
                    vec![],
                    auth_prob,
                    &reason,
                    line_num,
                );
            }
        }

        // Test common endpoints dynamically using the fetcher
        let common_paths = vec![
            "/swagger.json",
            "/openapi.json",
            "/api-docs",
            "/api/v1/health",
        ];
        for path in common_paths {
            if let Ok(test_url) = actual_base_url.join(path) {
                if fetcher.test_endpoint(test_url.as_str()).await {
                    let snippet = format!(
                        "Automated brute-force active ping returned 200 OK for: {}",
                        path
                    );
                    self.register_path(
                        &mut endpoints_map,
                        path,
                        &actual_base_url,
                        "Common Endpoint Brute-force",
                        &snippet,
                        "GET",
                        vec![],
                        0.0,
                        "Brute force successful status code 200",
                        None,
                    );
                    if path.contains("swagger") || path.contains("api-docs") {
                        tech_stack.insert("Swagger/OpenAPI".to_string());
                    }
                }
            }
        }

        // Output Formatting
        let mut endpoints_vec: Vec<EndpointDetail> = endpoints_map.into_values().collect();
        endpoints_vec.sort_by(|a, b| a.path.cmp(&b.path));

        let mut techs_vec: Vec<String> = tech_stack.into_iter().collect();
        techs_vec.sort();

        // Automated Verification and Metrics calculation
        let mut valid_count = 0;
        let mut false_positives = 0;
        let mut source_counts: HashMap<String, usize> = HashMap::new();
        let mut high_conf_valid = 0;
        let mut high_conf_total = 0;
        let mut low_conf_valid = 0;
        let mut low_conf_total = 0;

        let mut filtered_endpoints = Vec::new();

        for mut ep in endpoints_vec {
            for ev in &ep.evidences {
                *source_counts.entry(ev.source_type.clone()).or_insert(0) += 1;
            }

            let base_str = actual_base_url.as_str().trim_end_matches('/');
            let path_str = ep.path.trim_start_matches('/');
            let full_url = if ep.path.starts_with("http") {
                ep.path.clone()
            } else {
                format!("{}/{}", base_str, path_str)
            };

            if let Some(verification) = fetcher
                .verify_endpoint_deep(&full_url, &ep.method_prediction)
                .await
            {
                let is_valid = verification.is_valid;

                if ep.confidence_score >= 0.70 {
                    high_conf_total += 1;
                    if is_valid {
                        high_conf_valid += 1;
                    }
                } else {
                    low_conf_total += 1;
                    if is_valid {
                        low_conf_valid += 1;
                    }
                }

                // Hard filter logic for generic base paths
                if !is_valid
                    && ep.parameters.is_empty()
                    && (ep.path == "/api"
                        || ep.path == "/api/"
                        || ep.path == "/v1"
                        || ep.path == "/api/v1")
                {
                    continue; // Purge completely
                }

                if is_valid {
                    valid_count += 1;
                    ep.confidence_score = (ep.confidence_score + 0.3).min(0.95); // Boost
                    ep.method_prediction = verification.best_method.clone(); // Calibrate method
                } else {
                    false_positives += 1;
                    ep.confidence_score = (ep.confidence_score * 0.5).max(0.1); // Slash confidence
                }

                ep.runtime_verification = Some(verification);
            } else {
                ep.runtime_verification = None;
            }

            // Assign certainty based on verification outcome
            ep.certainty = Some(
                if ep.runtime_verification.as_ref().is_some_and(|v| v.is_valid) {
                    CertaintyLevel::Certain
                } else if ep.confidence_score >= 0.7 {
                    CertaintyLevel::Likely
                } else if ep.confidence_score >= 0.4 {
                    CertaintyLevel::Uncertain
                } else {
                    CertaintyLevel::Unknown
                },
            );

            filtered_endpoints.push(ep);
        }

        let total_endpoints = filtered_endpoints.len();

        let precision = if (valid_count + false_positives) > 0 {
            valid_count as f32 / (valid_count + false_positives) as f32
        } else {
            0.0
        };

        let high_acc = if high_conf_total > 0 {
            high_conf_valid as f32 / high_conf_total as f32
        } else {
            0.0
        };
        let low_acc = if low_conf_total > 0 {
            low_conf_valid as f32 / low_conf_total as f32
        } else {
            0.0
        };
        let correlation = high_acc - low_acc;

        let total_evidences: usize = source_counts.values().sum();
        let mut source_distribution = std::collections::HashMap::new();
        if total_evidences > 0 {
            for (k, v) in source_counts {
                source_distribution.insert(k, (v as f32 / total_evidences as f32) * 100.0);
            }
        }

        use crate::domain::entities::DiscoveryMetrics;
        let metrics = DiscoveryMetrics {
            total_endpoints,
            valid_endpoints: valid_count,
            false_positives,
            precision,
            source_distribution,
            confidence_accuracy_correlation: correlation,
        };

        let discovery_certainty = if valid_count > 0 {
            Some(CertaintyNote {
                level: CertaintyLevel::Certain,
                reason: format!("{} endpoint doğrulandı", valid_count),
            })
        } else if !filtered_endpoints.is_empty() {
            Some(CertaintyNote {
                level: CertaintyLevel::Uncertain,
                reason: "Endpoint'ler tespit edildi ama hiçbiri runtime'da doğrulanamadı"
                    .to_string(),
            })
        } else {
            Some(CertaintyNote {
                level: CertaintyLevel::Unknown,
                reason: "Hiçbir API yapısı tespit edilemedi — sonuçlar güvenilir değil".to_string(),
            })
        };

        Ok(ApiDiscoveryResult {
            base_url: actual_base_url.to_string(),
            detected_endpoints: filtered_endpoints,
            suspected_api_technologies: techs_vec,
            metrics: Some(metrics),
            discovery_certainty,
        })
    }
}
