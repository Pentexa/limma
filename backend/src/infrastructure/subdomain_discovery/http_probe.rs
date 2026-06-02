use crate::domain::entities::{HttpProbeResult, RedirectChainEntry};
use std::time::Instant;

pub struct HttpProber;

impl HttpProber {
    pub async fn probe(domain: &str, client: &reqwest::Client) -> Option<HttpProbeResult> {
        // We'll try HTTPS first, then HTTP
        let protocols = vec!["https", "http"];
        
        for proto in protocols {
            let url = format!("{}://{}", proto, domain);
            let start = Instant::now();
            
            if let Ok(res) = client.get(&url).send().await {
                let status_code = res.status().as_u16();
                let headers = res.headers();
                
                let server = headers.get("server").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                let content_type = headers.get("content-type").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                
                let mut technologies = Vec::new();
                if let Some(s) = &server {
                    technologies.push(s.clone());
                }
                if let Some(powered_by) = headers.get("x-powered-by").and_then(|h| h.to_str().ok()) {
                    technologies.push(powered_by.to_string());
                }
                
                let mut redirect_chain = Vec::new();
                if res.url().as_str() != url && res.url().as_str() != format!("{}/", url) {
                     redirect_chain.push(RedirectChainEntry {
                         url: res.url().to_string(),
                         status_code: 301,
                     });
                }
                
                let body = res.text().await.unwrap_or_default();
                let title = Self::extract_title(&body);
                
                return Some(HttpProbeResult {
                    url,
                    status_code,
                    title,
                    server,
                    content_type,
                    technologies,
                    tls_issuer: None,
                    tls_subject: None,
                    redirect_chain,
                    response_time_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
        
        None
    }
    
    fn extract_title(html: &str) -> Option<String> {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                return Some(html[start + 7..start + end].trim().to_string());
            }
        }
        if let Some(start) = html.find("<TITLE>") {
            if let Some(end) = html[start..].find("</TITLE>") {
                return Some(html[start + 7..start + end].trim().to_string());
            }
        }
        None
    }
}
