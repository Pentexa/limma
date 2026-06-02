pub mod dom_executor;
pub mod oob_callback;
pub mod payload_store;
pub mod timing_analyzer;
pub mod graphql_analyzer;
pub mod cache_analyzer;
pub mod smuggling_analyzer;

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
    graphql_analyzer: graphql_analyzer::GraphqlAnalyzer,
    cache_analyzer: cache_analyzer::CacheAnalyzer,
    smuggling_analyzer: smuggling_analyzer::SmugglingAnalyzer,
}

impl Default for HttpBlindDetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpBlindDetectionEngine {
    pub fn new() -> Self {
        Self {
            oob_engine: oob_callback::OobCallbackEngine::new(),
            dom_engine: dom_executor::DomExecutorImpl::new(),
            timing_analyzer: timing_analyzer::TimingAnalyzer::new(),
            graphql_analyzer: graphql_analyzer::GraphqlAnalyzer::new(),
            cache_analyzer: cache_analyzer::CacheAnalyzer::new(),
            smuggling_analyzer: smuggling_analyzer::SmugglingAnalyzer::new(),
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
                BlindVulnType::XmlExternalEntity => {
                    // XXE uses OOB callback with XML entity payloads
                    self.oob_engine.detect_ssrf(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::XmlExternalEntity;
                            f
                        })
                        .collect()
                }
                BlindVulnType::InsecureDeserialization => {
                    // Deserialization payloads use OOB callbacks to confirm execution
                    self.oob_engine.detect_ssrf(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::InsecureDeserialization;
                            f
                        })
                        .collect()
                }
                BlindVulnType::SecondOrderInjection => {
                    // Second-order injection uses OOB to detect delayed execution
                    self.oob_engine.detect_ssrf(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::SecondOrderInjection;
                            f
                        })
                        .collect()
                }
                BlindVulnType::RaceCondition => {
                    // Race conditions use timing analysis with concurrent requests
                    self.timing_analyzer
                        .detect_time_based_sqli(target_url)
                        .await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::RaceCondition;
                            f.detection_method = BlindDetectionMethod::ConcurrentTesting;
                            f
                        })
                        .collect()
                }
                BlindVulnType::JwtNoneAlg => {
                    // JWT none-algorithm bypass uses timing to detect auth bypass
                    self.timing_analyzer
                        .detect_time_based_sqli(target_url)
                        .await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::JwtNoneAlg;
                            f
                        })
                        .collect()
                }
                BlindVulnType::BlindSqliErrorBased => {
                    // Error-based blind SQLi uses differential analysis
                    self.timing_analyzer.detect_boolean_sqli(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::BlindSqliErrorBased;
                            f
                        })
                        .collect()
                }
                BlindVulnType::RemoteFileInclusion => {
                    // RFI uses OOB callback to detect external file inclusions
                    self.oob_engine.detect_ssrf(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::RemoteFileInclusion;
                            f
                        })
                        .collect()
                }
                BlindVulnType::HostHeaderInjection => {
                    // Host Header Injection uses OOB callback via injected Host headers
                    self.oob_engine.detect_ssrf(target_url).await?
                        .into_iter()
                        .map(|mut f| {
                            f.vulnerability_type = BlindVulnType::HostHeaderInjection;
                            f
                        })
                        .collect()
                }
                BlindVulnType::GraphqlAbuse => {
                    self.graphql_analyzer.detect_graphql_abuse(target_url).await?
                }
                BlindVulnType::WebCacheDeception => {
                    self.cache_analyzer.detect_cache_deception(target_url).await?
                }
                BlindVulnType::HttpRequestSmuggling => {
                    self.smuggling_analyzer.detect_request_smuggling(target_url).await?
                }
            };
            findings.append(&mut results);
        }

        Ok(findings)
    }
}
