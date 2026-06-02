pub mod active;
pub mod confidence;
pub mod http_probe;
pub mod passive;
pub mod resolver;
pub mod wildcard;

use crate::domain::engine_config::EngineConfig;
use crate::domain::entities::{
    CertaintyLevel, CertaintyNote, SubdomainAsset, SubdomainDiscoveryMetrics,
    SubdomainDiscoveryResult, SubdomainSource, SubdomainStatus,
};
use crate::domain::repositories::SubdomainDiscoverer;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct HttpSubdomainDiscoverer {
    client: reqwest::Client,
}

impl Default for HttpSubdomainDiscoverer {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpSubdomainDiscoverer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl SubdomainDiscoverer for HttpSubdomainDiscoverer {
    async fn discover_subdomains(
        &self,
        domain: &str,
        config: &EngineConfig,
    ) -> Result<SubdomainDiscoveryResult, String> {
        let start_time = std::time::Instant::now();
        
        // 1. Wildcard DNS detection
        let wildcard_info = wildcard::WildcardDetector::detect(domain).await;
        
        let mut all_candidates: HashMap<String, Vec<SubdomainSource>> = HashMap::new();
        let mut passive_count = 0;
        let mut active_count = 0;
        
        // 2. Passive collection
        if config.subdomain_enable_crtsh {
            let crt_subs = passive::PassiveCollector::collect_from_crtsh(domain, &self.client).await;
            passive_count = crt_subs.len();
            for sub in crt_subs {
                all_candidates.entry(sub).or_default().push(SubdomainSource::CrtSh);
            }
        }
        
        // 3. Active enumeration
        if config.subdomain_enable_dns_bruteforce {
            let active_subs = active::ActiveEnumerator::enumerate(domain, config, &wildcard_info.resolved_ips).await;
            active_count = active_subs.len();
            for sub in active_subs {
                all_candidates.entry(sub).or_default().push(SubdomainSource::DnsWordlist);
            }
        }
        
        let total_candidates = all_candidates.len();
        let mut assets = Vec::new();
        let mut validated_count = 0;
        let mut wildcard_filtered_count = 0;
        let mut http_alive_count = 0;
        
        // Process each candidate
        for (sub, sources) in all_candidates {
            // 4. DNS resolution
            let dns_records = resolver::DnsResolver::resolve_all(&sub).await;
            
            // Extract A/AAAA records for IPs
            let mut resolved_ips = Vec::new();
            for rec in &dns_records {
                if rec.record_type == "A" || rec.record_type == "AAAA" {
                    resolved_ips.extend(rec.values.clone());
                }
            }
            
            // Re-verify wildcard filter
            let mut is_wildcard = false;
            for ip in &resolved_ips {
                if wildcard_info.resolved_ips.contains(ip) {
                    is_wildcard = true;
                    break;
                }
            }
            
            if is_wildcard {
                wildcard_filtered_count += 1;
                continue;
            }
            
            if dns_records.is_empty() {
                continue; // Skip unresolved
            }
            
            validated_count += 1;
            
            // 5. HTTP Probe
            let mut http_probe = None;
            if config.subdomain_http_probe {
                http_probe = http_probe::HttpProber::probe(&sub, &self.client).await;
                if http_probe.is_some() {
                    http_alive_count += 1;
                }
            }
            
            let mut asset = SubdomainAsset {
                asset: sub,
                asset_type: "subdomain".to_string(),
                sources,
                resolved_ips,
                dns_records,
                http_probe,
                http_status: None,
                technologies: Vec::new(),
                confidence: 0.0,
                status: SubdomainStatus::Unresolved,
                risk_tags: Vec::new(),
                last_seen: chrono::Utc::now().to_rfc3339(),
                certainty: None,
            };
            
            // 6. Confidence Scoring
            confidence::ConfidenceScorer::score(&mut asset, &wildcard_info.resolved_ips);
            
            assets.push(asset);
        }
        
        // Sort assets by confidence descending, then asset name
        assets.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
                .then(a.asset.cmp(&b.asset))
        });
        
        let metrics = SubdomainDiscoveryMetrics {
            total_candidates,
            validated_count,
            wildcard_filtered_count,
            duplicate_removed_count: 0, // Handled implicitly by HashMap
            http_alive_count,
            passive_source_count: passive_count,
            active_source_count: active_count,
            precision: if total_candidates > 0 {
                validated_count as f32 / total_candidates as f32
            } else {
                0.0
            },
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
        };
        
        let discovery_certainty = if validated_count > 0 {
            Some(CertaintyNote {
                level: CertaintyLevel::Certain,
                reason: format!("{} verified subdomains found.", validated_count),
            })
        } else {
            Some(CertaintyNote {
                level: CertaintyLevel::Unknown,
                reason: "No subdomains found.".to_string(),
            })
        };
        
        Ok(SubdomainDiscoveryResult {
            domain: domain.to_string(),
            wildcard_dns: wildcard_info,
            assets,
            metrics,
            discovery_certainty,
            scan_timestamp: chrono::Utc::now(),
        })
    }
}
