use hickory_resolver::{config::{ResolverConfig, ResolverOpts}, AsyncResolver};
use std::collections::HashSet;
use std::sync::Arc;
use futures::stream::{StreamExt, FuturesUnordered};
use tokio::sync::Semaphore;
use crate::domain::engine_config::EngineConfig;

pub struct ActiveEnumerator;

impl ActiveEnumerator {
    pub async fn enumerate(
        domain: &str, 
        config: &EngineConfig,
        wildcard_ips: &[String],
    ) -> HashSet<String> {
        let mut subdomains = HashSet::new();
        
        let wordlist = match config.subdomain_wordlist_size.as_str() {
            "small" => vec!["www", "mail", "ftp", "api", "dev", "staging", "admin", "test", "blog", "shop"],
            "medium" => vec![
                "www", "mail", "ftp", "api", "dev", "staging", "admin", "test", "blog", "shop",
                "cdn", "assets", "portal", "vpn", "git", "jenkins", "grafana", "prometheus", "k8s", "docker"
            ],
            "large" | "massive" => vec![
                "www", "mail", "ftp", "api", "dev", "staging", "admin", "test", "blog", "shop",
                "cdn", "assets", "portal", "vpn", "git", "jenkins", "grafana", "prometheus", "k8s", "docker",
                "cloud", "aws", "gcp", "azure", "jira", "confluence", "wiki", "docs", "support", "help", "metrics"
            ],
            _ => vec!["www", "api", "dev", "staging", "admin", "test"]
        };
        
        let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        let sem = Arc::new(Semaphore::new(config.subdomain_max_concurrent_dns));
        
        let mut futures = FuturesUnordered::new();
        
        for prefix in wordlist {
            let candidate = format!("{}.{}", prefix, domain);
            let resolver = resolver.clone();
            let sem = sem.clone();
            let rate_limiter = config.rate_limiter.clone();
            let wildcard_ips = wildcard_ips.to_vec();
            
            futures.push(tokio::spawn(async move {
                rate_limiter.wait().await;
                let _permit = sem.acquire().await.unwrap();
                if let Ok(response) = resolver.lookup_ip(&candidate).await {
                    let mut valid = false;
                    for ip in response.iter() {
                        let ip_str = ip.to_string();
                        if !wildcard_ips.contains(&ip_str) {
                            valid = true;
                            break;
                        }
                    }
                    if valid {
                        return Some(candidate);
                    }
                }
                None
            }));
        }
        
        while let Some(res) = futures.next().await {
            if let Ok(Some(sub)) = res {
                subdomains.insert(sub);
            }
        }
        
        subdomains
    }
}
