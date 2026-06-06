use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::time::Instant;

use crate::api::models::{DiscoverCertificatesRequest, DiscoverCertificatesResponse};
use crate::domain::engine_config::EngineConfig;
use crate::domain::entities::{SubdomainAsset, SubdomainSource, SubdomainStatus};
use crate::infrastructure::subdomain_discovery::HttpSubdomainDiscoverer;
use crate::infrastructure::subdomain_discovery::wildcard;

pub struct CertificateDiscoverer {
    client: reqwest::Client,
    subdomain_discoverer: Arc<HttpSubdomainDiscoverer>,
}

impl CertificateDiscoverer {
    pub fn new(subdomain_discoverer: Arc<HttpSubdomainDiscoverer>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            subdomain_discoverer,
        }
    }

    pub async fn discover(
        &self,
        req: DiscoverCertificatesRequest,
        config: &EngineConfig,
    ) -> DiscoverCertificatesResponse {
        let start_time = Instant::now();
        let domain = &req.domain;
        let mut warnings = Vec::new();

        let mut total_cert_names = 0;
        let mut wildcard_removed = 0;
        let mut out_of_scope_removed = 0;
        
        let mut candidates_map: HashMap<String, Vec<SubdomainSource>> = HashMap::new();
        let mut tags_map: HashMap<String, HashSet<String>> = HashMap::new();

        let url = format!("https://crt.sh/?q=%.{}&output=json", domain);
        
        if let Ok(res) = self.client.get(&url).send().await {
            if let Ok(text) = res.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(arr) = json.as_array() {
                        for item in arr {
                            if let Some(name_value) = item.get("name_value") {
                                if let Some(name_str) = name_value.as_str() {
                                    for sub in name_str.split('\n') {
                                        total_cert_names += 1;
                                        
                                        let sub = sub.trim().to_lowercase();
                                        
                                        if sub.starts_with("*.") {
                                            wildcard_removed += 1;
                                            continue;
                                        }
                                        
                                        if !sub.ends_with(domain) || sub == *domain {
                                            out_of_scope_removed += 1;
                                            continue;
                                        }
                                        
                                        candidates_map.entry(sub.clone()).or_default().push(SubdomainSource::CrtSh);
                                        
                                        // Tagging
                                        let tags = tags_map.entry(sub.clone()).or_default();
                                        tags.insert("from-certificate".to_string());
                                        tags.insert("historical-subdomain".to_string());
                                        
                                        if sub.contains("dev") {
                                            tags.insert("dev-candidate".to_string());
                                        }
                                        if sub.contains("staging") || sub.contains("stage") || sub.contains("stg") {
                                            tags.insert("staging-candidate".to_string());
                                        }
                                        if sub.contains("admin") {
                                            tags.insert("admin-candidate".to_string());
                                        }
                                        if sub.contains("legacy") || sub.contains("old") || sub.contains("bak") {
                                            tags.insert("legacy-candidate".to_string());
                                        }
                                        if sub.contains("api") || sub.contains("auth") || sub.contains("login") || sub.contains("sso") || sub.contains("vpn") || sub.contains("portal") {
                                            tags.insert("sensitive-service-candidate".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        warnings.push("crt.sh response was not a JSON array".to_string());
                    }
                } else {
                    warnings.push("Failed to parse crt.sh JSON response".to_string());
                }
            } else {
                warnings.push("Failed to read crt.sh response text".to_string());
            }
        } else {
            warnings.push("crt.sh query failed (network error or timeout)".to_string());
        }

        let unique_candidates = candidates_map.len();
        
        let (mut assets, validated_count, extra_wildcard_filtered, _) = if req.validate_assets && unique_candidates > 0 {
            // Re-use wildcard info
            let wildcard_info = wildcard::WildcardDetector::detect(domain).await;
            
            self.subdomain_discoverer.validate_candidates(
                candidates_map,
                &wildcard_info,
                config,
            ).await
        } else {
            // If validation is disabled or no candidates, just map them to Unresolved assets
            let mut unvalidated_assets = Vec::new();
            for (sub, sources) in candidates_map {
                let asset = SubdomainAsset {
                    asset: sub,
                    asset_type: "subdomain".to_string(),
                    sources,
                    resolved_ips: Vec::new(),
                    dns_records: Vec::new(),
                    http_probe: None,
                    http_status: None,
                    technologies: Vec::new(),
                    confidence: 0.0, // No confidence score without validation
                    status: SubdomainStatus::Unresolved,
                    risk_tags: Vec::new(),
                    last_seen: chrono::Utc::now().to_rfc3339(),
                    certainty: None,
                };
                unvalidated_assets.push(asset);
            }
            (unvalidated_assets, 0, 0, 0)
        };
        
        wildcard_removed += extra_wildcard_filtered;
        
        // Inject tags into assets
        for asset in &mut assets {
            if let Some(tags) = tags_map.get(&asset.asset) {
                for tag in tags {
                    if !asset.risk_tags.contains(tag) {
                        asset.risk_tags.push(tag.clone());
                    }
                }
            }
        }

        DiscoverCertificatesResponse {
            total_cert_names,
            unique_candidates,
            wildcard_removed,
            out_of_scope_removed,
            validated_assets: validated_count,
            assets,
            source: "crt.sh".to_string(),
            warnings,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}
