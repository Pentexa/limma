pub mod cmdi_detector;
pub mod deser_detector;
pub mod idor_detector;
pub mod jwt_detector;
pub mod lfi_detector;
pub mod nosql_detector;
pub mod redirect_detector;
pub mod sqli_detector;
pub mod ssrf_detector;
pub mod ssti_detector;
pub mod xss_detector;
pub mod xxe_detector;

use crate::domain::active_vuln::{ActiveVulnFinding, ActiveVulnType};
use crate::domain::fuzzing::{EndpointContext, InsertionPoint};
use crate::infrastructure::active_detection::differential::BaselineProfile;
use crate::infrastructure::safety::waf_monitor::WafMonitor;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct PayloadResponse {
    pub request_raw: String,
    pub response_body: String,
    pub response_time_ms: u64,
    pub status_code: u16,
    pub request_url: String,
    pub http_method: String,
    pub browser_verification_url: Option<String>,
    pub headers: reqwest::header::HeaderMap,
}

pub async fn send_payload_request(
    client: &reqwest::Client,
    target_url: &str,
    parameter: &str,
    payload: &str,
    endpoint_ctx: Option<&EndpointContext>,
    insertion_point: Option<&InsertionPoint>,
    apply_bypass_headers: bool,
) -> Result<PayloadResponse, String> {
    let start = std::time::Instant::now();

    let (mut request, request_url, http_method, request_raw, browser_verification_url) = if let (
        Some(ctx),
        Some(point),
    ) =
        (endpoint_ctx, insertion_point)
    {
        let allowed_domains = url::Url::parse(target_url)
            .ok()
            .and_then(|url| url.host_str().map(ToString::to_string))
            .into_iter()
            .collect();
        let replayer = crate::infrastructure::active_detection::fuzzing::request_replayer::RequestReplayer::new(
                client.clone(),
                None,
                allowed_domains,
                true,
            );
        let request = replayer.build_request(ctx, point, payload)?;
        let browser_url = if ctx.method.eq_ignore_ascii_case("GET") {
            if let InsertionPoint::QueryParam(param) = point {
                Some(append_query_param(&ctx.url, param, payload))
            } else {
                None
            }
        } else {
            None
        };
        (
            request,
            ctx.url.clone(),
            ctx.method.to_uppercase(),
            format!(
                "{} {} HTTP/1.1\nInsertion-Point: {:?}\nPayload: {}",
                ctx.method.to_uppercase(),
                ctx.url,
                point,
                payload
            ),
            browser_url,
        )
    } else {
        let test_url = append_query_param(target_url, parameter, payload);
        (
            client.get(&test_url),
            test_url.clone(),
            "GET".to_string(),
            format!("GET {} HTTP/1.1", test_url),
            Some(test_url),
        )
    };

    if apply_bypass_headers {
        request =
            crate::infrastructure::active_detection::waf_bypass_headers::apply_waf_bypass(request);
    }

    let resp = request.send().await.map_err(|e| e.to_string())?;
    let status_code = resp.status().as_u16();
    let headers = resp.headers().clone();
    let response_body = resp.text().await.map_err(|e| e.to_string())?;
    let response_time_ms = start.elapsed().as_millis() as u64;

    Ok(PayloadResponse {
        request_raw,
        response_body,
        response_time_ms,
        status_code,
        request_url,
        http_method,
        browser_verification_url,
        headers,
    })
}

pub fn append_query_param(target_url: &str, parameter: &str, payload: &str) -> String {
    if let Ok(mut url) = url::Url::parse(target_url) {
        url.query_pairs_mut().append_pair(parameter, payload);
        url.to_string()
    } else {
        let separator = if target_url.contains('?') { '&' } else { '?' };
        format!(
            "{}{}{}={}",
            target_url,
            separator,
            parameter,
            urlencoding::encode(payload)
        )
    }
}

/// Common trait for all vulnerability detectors.
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait VulnDetector: Send + Sync {
    /// Returns which vulnerability types this detector can identify.
    fn supported_types(&self) -> Vec<ActiveVulnType>;

    /// Runs detection against a specific target URL and parameter.
    async fn detect(
        &self,
        target_url: &str,
        parameter: &str,
        scan_id: Uuid,
        payload_selector: &crate::infrastructure::active_detection::payload_selector::PayloadSelector,
        rate_limit_ms: u64,
        waf_monitor: Arc<WafMonitor>,
        baseline: Option<&BaselineProfile>,
        endpoint_ctx: Option<&EndpointContext>,
        insertion_point: Option<&InsertionPoint>,
    ) -> Result<Vec<ActiveVulnFinding>, String>;
}

/// Factory function to build all detectors with a specific HTTP client.
/// This allows each scan to have its own authenticated client instance.
pub fn build_detectors(client: reqwest::Client) -> Vec<Box<dyn VulnDetector>> {
    vec![
        Box::new(xss_detector::XssDetector::new(client.clone())),
        Box::new(sqli_detector::SqliDetector::new(client.clone())),
        Box::new(cmdi_detector::CmdiDetector::new(client.clone())),
        Box::new(lfi_detector::LfiDetector::new(client.clone())),
        Box::new(ssrf_detector::SsrfDetector::new(client.clone())),
        Box::new(xxe_detector::XxeDetector::new(client.clone())),
        Box::new(redirect_detector::RedirectDetector::new(client.clone())),
        Box::new(jwt_detector::JwtDetector::new(client.clone())),
        Box::new(deser_detector::DeserDetector::new(client.clone())),
        Box::new(idor_detector::IdorDetector::new(client.clone())),
        Box::new(nosql_detector::NosqlDetector::new(client.clone())),
        Box::new(ssti_detector::SstiDetector::new(client.clone())),
    ]
}
