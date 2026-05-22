pub mod dom_executor;
pub mod oob_callback;
pub mod payload_store;
pub mod timing_analyzer;

use crate::domain::entities::*;
use async_trait::async_trait;

/// Infrastructure trait for blind vulnerability detection
#[async_trait]
pub trait BlindDetectionEngine: Send + Sync {
    async fn detect_blind_vulnerabilities(
        &self,
        target_url: &str,
        types: &[BlindVulnType],
    ) -> Result<Vec<RawBlindFinding>, String>;
}

/// Concrete implementation combining multiple blind detection strategies
pub struct HttpBlindDetectionEngine {
    oob_engine: oob_callback::OobCallbackEngine,
    dom_engine: dom_executor::DomExecutorImpl,
    timing_analyzer: timing_analyzer::TimingAnalyzer,
}

impl HttpBlindDetectionEngine {
    pub fn new() -> Self {
        Self {
            oob_engine: oob_callback::OobCallbackEngine::new(),
            dom_engine: dom_executor::DomExecutorImpl::new(),
            timing_analyzer: timing_analyzer::TimingAnalyzer::new(),
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
            let mut results = match vuln_type {
                BlindVulnType::BlindSqliTimeBased => {
                    self.timing_analyzer
                        .detect_time_based_sqli(target_url)
                        .await?
                }
                BlindVulnType::BlindSqliBoolean => {
                    self.timing_analyzer.detect_boolean_sqli(target_url).await?
                }
                BlindVulnType::DomXss => self.dom_engine.detect_dom_xss(target_url).await?,
                BlindVulnType::BlindSsrfDns | BlindVulnType::BlindSsrfHttp => {
                    self.oob_engine.detect_ssrf(target_url).await?
                }
                _ => {
                    tracing::debug!("Blind detection not yet implemented for {:?}", vuln_type);
                    vec![]
                }
            };
            findings.append(&mut results);
        }

        Ok(findings)
    }
}
