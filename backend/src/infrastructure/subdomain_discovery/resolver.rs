use hickory_resolver::{config::{ResolverConfig, ResolverOpts}, AsyncResolver};
use crate::domain::entities::DnsRecord;

pub struct DnsResolver;

impl DnsResolver {
    pub async fn resolve_all(domain: &str) -> Vec<DnsRecord> {
        let mut records = Vec::new();
        let resolver = AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        
        // A
        if let Ok(response) = resolver.ipv4_lookup(domain).await {
            let vals: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();
            if !vals.is_empty() {
                records.push(DnsRecord { record_type: "A".to_string(), values: vals });
            }
        }
        // AAAA
        if let Ok(response) = resolver.ipv6_lookup(domain).await {
            let vals: Vec<String> = response.iter().map(|ip| ip.to_string()).collect();
            if !vals.is_empty() {
                records.push(DnsRecord { record_type: "AAAA".to_string(), values: vals });
            }
        }
        // CNAME
        if let Ok(response) = resolver.lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME).await {
            let vals: Vec<String> = response.iter().map(|r| r.to_string()).collect();
            if !vals.is_empty() {
                records.push(DnsRecord { record_type: "CNAME".to_string(), values: vals });
            }
        }
        // MX
        if let Ok(response) = resolver.mx_lookup(domain).await {
            let vals: Vec<String> = response.iter().map(|mx| mx.exchange().to_string()).collect();
            if !vals.is_empty() {
                records.push(DnsRecord { record_type: "MX".to_string(), values: vals });
            }
        }
        // TXT
        if let Ok(response) = resolver.txt_lookup(domain).await {
            let vals: Vec<String> = response.iter().map(|txt| txt.to_string()).collect();
            if !vals.is_empty() {
                records.push(DnsRecord { record_type: "TXT".to_string(), values: vals });
            }
        }
        
        records
    }
}
