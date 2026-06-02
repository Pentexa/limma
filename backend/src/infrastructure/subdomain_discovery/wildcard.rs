use hickory_resolver::{config::{ResolverConfig, ResolverOpts}, AsyncResolver};
use uuid::Uuid;

pub struct WildcardDetector;

impl WildcardDetector {
    pub async fn detect(domain: &str) -> crate::domain::entities::WildcardDnsInfo {
        let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        
        let random_prefix = format!("limma-test-{}", Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect::<String>());
        let test_domain = format!("{}.{}", random_prefix, domain);
        
        let mut resolved_ips = Vec::new();
        if let Ok(response) = resolver.lookup_ip(&test_domain).await {
            for ip in response.iter() {
                resolved_ips.push(ip.to_string());
            }
        }
        
        crate::domain::entities::WildcardDnsInfo {
            is_wildcard: !resolved_ips.is_empty(),
            test_subdomain: test_domain,
            resolved_ips,
        }
    }
}
