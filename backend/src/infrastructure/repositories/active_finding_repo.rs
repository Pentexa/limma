#![allow(clippy::type_complexity, clippy::needless_borrows_for_generic_args)]
use crate::domain::active_vuln::ExploitabilityLevel;
use crate::domain::active_vuln::{ActiveVulnFinding, ActiveVulnType};
use crate::domain::entities::{ConfidenceLevel, SeverityLevel};
use crate::domain::repositories::ActiveFindingRepository;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PgActiveFindingRepository {
    pool: PgPool,
}

impl PgActiveFindingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_finding(
        row: (
            uuid::Uuid,
            uuid::Uuid,
            chrono::DateTime<chrono::Utc>,
            String,
            String,
            String,
            String,
            String,
            serde_json::Value,
            String,
            String,
            String,
            bool,
            Option<uuid::Uuid>,
            bool,
            bool,
        ),
    ) -> Result<ActiveVulnFinding, String> {
        let vuln_type: ActiveVulnType = serde_json::from_value(serde_json::Value::String(row.3))
            .map_err(|e| format!("Invalid vuln_type: {}", e))?;
        let severity: SeverityLevel =
            serde_json::from_value(serde_json::Value::String(row.9)).unwrap_or(SeverityLevel::Low);
        let confidence: ConfidenceLevel = serde_json::from_value(serde_json::Value::String(row.10))
            .unwrap_or(ConfidenceLevel::Tentative);
        let exploitability: ExploitabilityLevel =
            serde_json::from_value(serde_json::Value::String(row.11))
                .unwrap_or(ExploitabilityLevel::Theoretical);
        let evidence =
            serde_json::from_value(row.8).map_err(|e| format!("Invalid evidence JSON: {}", e))?;

        Ok(ActiveVulnFinding {
            id: row.0,
            scan_id: row.1,
            timestamp: row.2,
            vuln_type,
            target_url: row.4,
            affected_parameter: row.5,
            http_method: row.6,
            payload_used: row.7,
            evidence,
            severity,
            confidence,
            exploitability,
            poc_generated: row.12,
            poc_id: row.13,
            verified: row.14,
            false_positive: row.15,
        })
    }
}

#[async_trait]
impl ActiveFindingRepository for PgActiveFindingRepository {
    async fn save_finding(&self, finding: ActiveVulnFinding) -> Result<(), String> {
        let evidence_json = serde_json::to_value(&finding.evidence)
            .map_err(|e| format!("Failed to serialize evidence: {}", e))?;
        let vuln_type_str = serde_json::to_value(&finding.vuln_type)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let severity_str = serde_json::to_value(&finding.severity)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let confidence_str = serde_json::to_value(&finding.confidence)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let expl_str = serde_json::to_value(&finding.exploitability)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO active_findings (
                id, scan_id, timestamp, vuln_type, target_url, affected_parameter,
                http_method, payload_used, evidence, severity, confidence,
                exploitability, poc_generated, poc_id, verified, false_positive
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(finding.id)
        .bind(finding.scan_id)
        .bind(finding.timestamp)
        .bind(vuln_type_str)
        .bind(&finding.target_url)
        .bind(&finding.affected_parameter)
        .bind(&finding.http_method)
        .bind(&finding.payload_used)
        .bind(&evidence_json)
        .bind(severity_str)
        .bind(confidence_str)
        .bind(expl_str)
        .bind(finding.poc_generated)
        .bind(finding.poc_id)
        .bind(finding.verified)
        .bind(finding.false_positive)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error inserting active finding: {}", e))?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        finding_id: uuid::Uuid,
    ) -> Result<Option<ActiveVulnFinding>, String> {
        let row = sqlx::query_as(
            r#"
            SELECT id, scan_id, timestamp, vuln_type, target_url, affected_parameter,
                   http_method, payload_used, evidence, severity, confidence,
                   exploitability, poc_generated, poc_id, verified, false_positive
            FROM active_findings WHERE id = $1
            "#,
        )
        .bind(finding_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching finding: {}", e))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_finding(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_scan_id(&self, scan_id: uuid::Uuid) -> Result<Vec<ActiveVulnFinding>, String> {
        let rows = sqlx::query_as(
            r#"
            SELECT id, scan_id, timestamp, vuln_type, target_url, affected_parameter,
                   http_method, payload_used, evidence, severity, confidence,
                   exploitability, poc_generated, poc_id, verified, false_positive
            FROM active_findings WHERE scan_id = $1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(scan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching scan findings: {}", e))?;

        let mut findings = Vec::new();
        for r in rows {
            findings.push(Self::row_to_finding(r)?);
        }
        Ok(findings)
    }

    async fn update_poc_id(
        &self,
        finding_id: uuid::Uuid,
        poc_id: uuid::Uuid,
    ) -> Result<(), String> {
        sqlx::query("UPDATE active_findings SET poc_id = $1, poc_generated = true WHERE id = $2")
            .bind(poc_id)
            .bind(finding_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error updating poc_id: {}", e))?;
        Ok(())
    }

    async fn find_by_filters(
        &self,
        params: &crate::domain::active_vuln::ActiveFindingQueryParams,
    ) -> Result<Vec<ActiveVulnFinding>, String> {
        let mut builder = sqlx::QueryBuilder::new(
            "SELECT id, scan_id, timestamp, vuln_type, target_url, affected_parameter, http_method, payload_used, evidence, severity, confidence, exploitability, poc_generated, poc_id, verified, false_positive FROM active_findings WHERE 1=1"
        );

        if let Some(scan_id) = params.scan_id {
            builder.push(" AND scan_id = ");
            builder.push_bind(scan_id);
        }

        if let Some(ref vuln_type) = params.vuln_type {
            let vuln_str = serde_json::to_value(vuln_type)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            builder.push(" AND vuln_type = ");
            builder.push_bind(vuln_str);
        }

        if let Some(ref severity) = params.severity {
            let sev_str = serde_json::to_value(severity)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            builder.push(" AND severity = ");
            builder.push_bind(sev_str);
        }

        builder.push(" ORDER BY timestamp DESC");

        let rows = builder
            .build_query_as::<(
                uuid::Uuid,
                uuid::Uuid,
                chrono::DateTime<chrono::Utc>,
                String,
                String,
                String,
                String,
                String,
                serde_json::Value,
                String,
                String,
                String,
                bool,
                Option<uuid::Uuid>,
                bool,
                bool,
            )>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error fetching filtered findings: {}", e))?;

        let mut findings = Vec::new();
        for r in rows {
            findings.push(Self::row_to_finding(r)?);
        }
        Ok(findings)
    }

    async fn update_status(
        &self,
        finding_id: uuid::Uuid,
        verified: bool,
        false_positive: bool,
    ) -> Result<(), String> {
        sqlx::query("UPDATE active_findings SET verified = $1, false_positive = $2 WHERE id = $3")
            .bind(verified)
            .bind(false_positive)
            .bind(finding_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error updating finding status: {}", e))?;
        Ok(())
    }

    async fn delete_by_scan_id(&self, scan_id: uuid::Uuid) -> Result<u64, String> {
        let result = sqlx::query("DELETE FROM active_findings WHERE scan_id = $1")
            .bind(scan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error deleting findings by scan_id: {}", e))?;
        Ok(result.rows_affected())
    }
}
