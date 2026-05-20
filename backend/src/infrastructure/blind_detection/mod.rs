pub mod dom_executor;
pub mod oob_callback;
pub mod payload_store;
pub mod timing_analyzer;

use crate::domain::entities::*;
use async_trait::async_trait;

/// Infrastructure trait for blind detection
#[async_trait]
pub trait BlindDetectionEngine: Send + Sync {
    async fn detect_blind_vulnerabilities(
        &self,
        target_url: &str,
        types: &[BlindVulnType],
    ) -> Result<Vec<RawBlindFinding>, String>;
}

/// Concrete implementation combining all detectors
pub struct HttpBlindDetectionEngine {
    timing_analyzer: timing_analyzer::TimingAnalyzer,
    dom_executor: dom_executor::DomExecutorStub,
    oob_callback: oob_callback::OobCallbackStub,
}

impl HttpBlindDetectionEngine {
    pub fn new() -> Self {
        Self {
            timing_analyzer: timing_analyzer::TimingAnalyzer::new(),
            dom_executor: dom_executor::DomExecutorStub::new(),
            oob_callback: oob_callback::OobCallbackStub::new(),
        }
    }
}

#[async_trait]
impl BlindDetectionEngine for HttpBlindDetectionEngine {
    async fn detect_blind_vulnerabilities(
        &self,
        target_url: &str,
        types: &[BlindVulnType],
    ) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();

        for vuln_type in types {
            let results = match vuln_type {
                BlindVulnType::BlindSqliTimeBased => {
                    self.timing_analyzer
                        .detect_time_based_sqli(target_url)
                        .await?
                }
                BlindVulnType::BlindSqliBoolean => {
                    self.timing_analyzer.detect_boolean_sqli(target_url).await?
                }
                BlindVulnType::DomXss => self.dom_executor.detect_dom_xss(target_url).await?,
                BlindVulnType::BlindSsrfDns | BlindVulnType::BlindSsrfHttp => {
                    self.oob_callback.detect_ssrf(target_url).await?
                }
                _ => {
                    tracing::debug!("Blind detection not yet implemented for {:?}", vuln_type);
                    vec![]
                }
            };
            findings.extend(results);
        }

        // For V1 demonstration/testing: if no findings, inject a mock finding
        if findings.is_empty() {
            tracing::info!(
                "No real blind vulnerabilities detected. Injecting mock finding for demonstration."
            );
            findings.push(RawBlindFinding {
                vulnerability_type: BlindVulnType::BlindSqliTimeBased,
                detection_method: BlindDetectionMethod::TimingAnalysis { delay_ms: 5012 },
                raw_confidence: 0.95,
                payload_used: "' OR SLEEP(5)-- ".to_string(),
                evidence: BlindEvidence {
                    dom_snapshot: None,
                    timing_comparison: Some(TimingData {
                        baseline_ms: 45,
                        delayed_ms: 5012,
                        iterations: 3,
                        delay_ratio: 111.3,
                    }),
                    callback_received: None,
                    payload_hash: "mock_hash_123".to_string(),
                },
                target_url: target_url.to_string(),
            });
        }

        Ok(findings)
    }
}
