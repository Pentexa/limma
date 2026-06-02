use crate::domain::entities::{SubdomainAsset, SubdomainSource, SubdomainStatus, CertaintyLevel};

pub struct ConfidenceScorer;

impl ConfidenceScorer {
    pub fn score(asset: &mut SubdomainAsset, wildcard_ips: &[String]) {
        let mut score: f32 = 0.0;
        let mut tags = Vec::new();
        
        // Base score from sources
        if asset.sources.len() > 1 {
            score += 0.60;
        } else if asset.sources.contains(&SubdomainSource::DnsWordlist) {
            score += 0.50;
        } else {
            score += 0.40; // Only passive
        }
        
        // DNS Records
        if !asset.dns_records.is_empty() {
            score += 0.20;
            
            // Check for CNAME dangling
            let has_cname = asset.dns_records.iter().any(|r| r.record_type == "CNAME");
            let has_a = asset.dns_records.iter().any(|r| r.record_type == "A" || r.record_type == "AAAA");
            if has_cname && !has_a {
                tags.push("cname-dangling".to_string());
            }
            
            let a_records = asset.dns_records.iter().find(|r| r.record_type == "A");
            if let Some(a_rec) = a_records {
                if a_rec.values.len() > 1 {
                    tags.push("multi-ip".to_string());
                }
            }
        }
        
        // HTTP Probe
        if let Some(probe) = &asset.http_probe {
            score += 0.25;
            if probe.url.starts_with("https") {
                score += 0.10;
            }
            tags.push("public-facing".to_string());
            asset.status = SubdomainStatus::HttpAlive;
            asset.http_status = Some(probe.status_code);
            asset.technologies.extend(probe.technologies.clone());
        } else if !asset.dns_records.is_empty() {
            tags.push("internal-candidate".to_string());
            asset.status = SubdomainStatus::Validated; // Validated via DNS but no HTTP
        } else {
            asset.status = SubdomainStatus::Unresolved;
        }
        
        // Wildcard check (just in case it slipped through)
        if asset.resolved_ips.iter().any(|ip| wildcard_ips.contains(ip)) {
            score -= 0.40;
            asset.status = SubdomainStatus::WildcardFiltered;
        }
        
        asset.confidence = score.clamp(0.0, 1.0);
        asset.risk_tags = tags;
        
        asset.certainty = Some(
            if asset.confidence >= 0.8 { CertaintyLevel::Certain }
            else if asset.confidence >= 0.5 { CertaintyLevel::Likely }
            else { CertaintyLevel::Uncertain }
        );
        
        // Deduplicate technologies
        asset.technologies.sort();
        asset.technologies.dedup();
    }
}
