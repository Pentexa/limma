use crate::domain::entities::*;
use crate::domain::repositories::BlindFindingRepository;
use async_trait::async_trait;
use uuid::Uuid;

/// PostgreSQL implementation of BlindFindingRepository
pub struct PgBlindFindingRepository {
    pool: sqlx::PgPool,
}

impl PgBlindFindingRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BlindFindingRepository for PgBlindFindingRepository {
    async fn save(&self, finding: &BlindFinding) -> Result<(), String> {
        let evidence_json = serde_json::to_value(&finding.evidence).map_err(|e| e.to_string())?;
        let vuln_type_str = format!("{:?}", finding.vulnerability_type);
        let detection_method_str = format!("{:?}", finding.detection_method);

        sqlx::query(
            r#"INSERT INTO blind_findings (id, scan_id, target_id, vulnerability_type, detection_method, confidence, evidence, payload_used, created_at, verified)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(finding.id)
        .bind(finding.scan_id)
        .bind(finding.target_id)
        .bind(&vuln_type_str)
        .bind(&detection_method_str)
        .bind(finding.confidence)
        .bind(&evidence_json)
        .bind(&finding.payload_used)
        .bind(finding.created_at)
        .bind(finding.verified)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save blind finding: {}", e))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BlindFinding>, String> {
        let row = sqlx::query_as::<_, BlindFindingRow>(
            "SELECT id, scan_id, target_id, vulnerability_type, detection_method, confidence, evidence, payload_used, created_at, verified FROM blind_findings WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find blind finding: {}", e))?;

        match row {
            Some(r) => Ok(Some(r.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn find_by_scan(&self, scan_id: Uuid) -> Result<Vec<BlindFinding>, String> {
        let rows = sqlx::query_as::<_, BlindFindingRow>(
            "SELECT id, scan_id, target_id, vulnerability_type, detection_method, confidence, evidence, payload_used, created_at, verified FROM blind_findings WHERE scan_id = $1 ORDER BY created_at DESC",
        )
        .bind(scan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find blind findings by scan: {}", e))?;

        rows.into_iter().map(|r| r.into_entity()).collect()
    }

    async fn update_verification(&self, id: Uuid, verified: bool) -> Result<(), String> {
        sqlx::query("UPDATE blind_findings SET verified = $1 WHERE id = $2")
            .bind(verified)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update verification: {}", e))?;

        Ok(())
    }
}

/// Internal row type for sqlx query mapping
#[derive(sqlx::FromRow)]
struct BlindFindingRow {
    id: Uuid,
    scan_id: Uuid,
    target_id: Uuid,
    vulnerability_type: String,
    
    
    confidence: f32,
    evidence: serde_json::Value,
    payload_used: String,
    created_at: chrono::DateTime<chrono::Utc>,
    verified: bool,
}

impl BlindFindingRow {
    fn into_entity(self) -> Result<BlindFinding, String> {
        let vulnerability_type: BlindVulnType = serde_json::from_str(&format!(
            "\"{}\"",
            self.vulnerability_type.to_lowercase().replace("::", "_")
        ))
        .unwrap_or(BlindVulnType::BlindSqliTimeBased);

        let detection_method: BlindDetectionMethod =
            serde_json::from_value(serde_json::json!({"differential_analysis": {}}))
                .unwrap_or(BlindDetectionMethod::DifferentialAnalysis);

        let evidence: BlindEvidence =
            serde_json::from_value(self.evidence).map_err(|e| e.to_string())?;

        Ok(BlindFinding {
            id: self.id,
            scan_id: self.scan_id,
            target_id: self.target_id,
            target_url: "".to_string(),
            vulnerable_parameter: None,
            vulnerability_type,
            detection_method,
            confidence: self.confidence,
            evidence,
            payload_used: self.payload_used,
            created_at: self.created_at,
            verified: self.verified,
        })
    }
}
