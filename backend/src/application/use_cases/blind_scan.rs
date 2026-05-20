use crate::domain::entities::*;
use crate::domain::repositories::BlindFindingRepository;
use crate::domain::services::BlindDetectionScoringService;
use crate::infrastructure::blind_detection::BlindDetectionEngine;
use crate::infrastructure::safety::SafetyFramework;
use std::time::Instant;
use uuid::Uuid;

/// Input DTO for blind scan request
#[derive(Debug, Clone)]
pub struct BlindScanRequest {
    pub scan_id: Uuid,
    pub target_url: String,
    pub target_id: Uuid,
    pub detection_types: Vec<BlindVulnType>,
    #[allow(dead_code)]
    pub max_duration_seconds: u32,
}

/// Output DTO for blind scan result
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlindScanResult {
    pub scan_id: Uuid,
    pub findings: Vec<BlindFinding>,
    pub duration_ms: u64,
    pub detection_summary: DetectionSummary,
}

/// Use case: Perform blind vulnerability detection
pub struct PerformBlindScan<'a> {
    pub finding_repo: &'a dyn BlindFindingRepository,
    pub safety_framework: &'a dyn SafetyFramework,
    pub detection_engine: &'a dyn BlindDetectionEngine,
}

impl<'a> PerformBlindScan<'a> {
    pub async fn execute(&self, request: BlindScanRequest) -> Result<BlindScanResult, String> {
        // 1. Validate scope
        self.safety_framework
            .validate_target(&request.target_url)
            .await
            .map_err(|e| format!("Safety validation failed: {}", e))?;

        // 2. Execute detection (delegates to infrastructure)
        let start = Instant::now();
        let raw_findings = self
            .detection_engine
            .detect_blind_vulnerabilities(&request.target_url, &request.detection_types)
            .await
            .map_err(|e| format!("Blind detection failed: {}", e))?;

        // 3. Apply domain scoring
        let scored_findings: Vec<BlindFinding> = raw_findings
            .into_iter()
            .map(|f| self.apply_scoring(f, request.scan_id, request.target_id))
            .collect();

        // 4. Persist findings
        for finding in &scored_findings {
            self.finding_repo.save(finding).await?;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(BlindScanResult {
            scan_id: request.scan_id,
            detection_summary: DetectionSummary::from_findings(&scored_findings, duration_ms),
            findings: scored_findings,
            duration_ms,
        })
    }

    fn apply_scoring(
        &self,
        finding: RawBlindFinding,
        scan_id: Uuid,
        target_id: Uuid,
    ) -> BlindFinding {
        // Delegate to domain service for timing-based confidence calculation
        let confidence = match &finding.detection_method {
            BlindDetectionMethod::TimingAnalysis { delay_ms } => {
                BlindDetectionScoringService::calculate_timing_confidence(
                    100, // baseline assumption
                    *delay_ms, 3, // default iterations
                )
            }
            _ => finding.raw_confidence,
        };

        finding.to_entity(scan_id, target_id, confidence)
    }
}
