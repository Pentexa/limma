use reqwest::Client;
use std::time::Duration;

use crate::infrastructure::active_detection::differential::BaselineProfile;
use crate::infrastructure::active_detection::evidence::response_diff::stable_hash;

const BASELINE_SAMPLE_COUNT: usize = 2;

pub struct BaselineAnalyzer;

impl BaselineAnalyzer {
    pub async fn build(
        client: &Client,
        target_url: &str,
        param: &str,
        safe_value: &str,
    ) -> Result<BaselineProfile, String> {
        let mut samples = Vec::new();

        for _ in 0..BASELINE_SAMPLE_COUNT {
            samples.push(Self::capture_sample(client, target_url, param, safe_value).await?);
        }

        let first = samples
            .first()
            .ok_or_else(|| "baseline sampling returned no responses".to_string())?;
        let average_response_time_ms = samples
            .iter()
            .map(|sample| sample.response_time_ms)
            .sum::<u64>()
            / samples.len() as u64;
        let error_rate = samples
            .iter()
            .filter(|sample| sample.status_code >= 500)
            .count() as f32
            / samples.len() as f32;

        Ok(BaselineProfile {
            status_code: first.status_code,
            content_length: first.response_body.len(),
            response_body: first.response_body.clone(),
            response_time_ms: average_response_time_ms,
            average_response_time_ms,
            body_hash: stable_hash(&first.response_body),
            header_fingerprint: first.header_fingerprint.clone(),
            error_rate,
            redirect_location: first.redirect_location.clone(),
        })
    }

    async fn capture_sample(
        client: &Client,
        target_url: &str,
        param: &str,
        safe_value: &str,
    ) -> Result<BaselineSample, String> {
        let start_time = std::time::Instant::now();
        let resp = client
            .get(target_url)
            .query(&[(param, safe_value)])
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let elapsed = start_time.elapsed().as_millis() as u64;
        let status_code = resp.status().as_u16();
        let headers = resp.headers().clone();
        let redirect_location = headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let header_fingerprint = headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| format!("{}:{}", name.as_str().to_lowercase(), value))
            })
            .collect();
        let response_body = resp.text().await.unwrap_or_default();

        Ok(BaselineSample {
            status_code,
            response_time_ms: elapsed,
            response_body,
            header_fingerprint,
            redirect_location,
        })
    }
}

struct BaselineSample {
    status_code: u16,
    response_time_ms: u64,
    response_body: String,
    header_fingerprint: Vec<String>,
    redirect_location: Option<String>,
}
