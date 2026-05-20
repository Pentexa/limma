#![allow(clippy::needless_borrows_for_generic_args)]
use crate::domain::active_vuln::{ActiveScanResult, ActiveScanStatus};
use crate::domain::repositories::ActiveScanRepository;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PgActiveScanRepository {
    pool: PgPool,
}

impl PgActiveScanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActiveScanRepository for PgActiveScanRepository {
    async fn create_scan(&self, scan: ActiveScanResult) -> Result<(), String> {
        let summary_json = serde_json::to_value(&scan.summary)
            .map_err(|e| format!("Failed to serialize summary: {}", e))?;
        let errors_json = serde_json::to_value(&scan.errors)
            .map_err(|e| format!("Failed to serialize errors: {}", e))?;
        let status_str = serde_json::to_value(&scan.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .as_str()
            .unwrap_or("pending")
            .to_string();

        sqlx::query(
            r#"
            INSERT INTO active_scans (id, target_url, status, start_time, end_time, total_requests, summary, errors)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(scan.scan_id)
        .bind(&scan.target_url)
        .bind(status_str)
        .bind(scan.start_time)
        .bind(scan.end_time)
        .bind(scan.total_requests as i32)
        .bind(&summary_json)
        .bind(&errors_json)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error inserting active scan: {}", e))?;

        Ok(())
    }

    async fn update_status(
        &self,
        scan_id: uuid::Uuid,
        status: ActiveScanStatus,
    ) -> Result<(), String> {
        let status_str = serde_json::to_value(&status)
            .unwrap_or_else(|_| serde_json::json!("pending"))
            .as_str()
            .unwrap_or("pending")
            .to_string();

        sqlx::query("UPDATE active_scans SET status = $1 WHERE id = $2")
            .bind(status_str)
            .bind(scan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error updating status: {}", e))?;

        Ok(())
    }

    async fn find_by_id(&self, scan_id: uuid::Uuid) -> Result<Option<ActiveScanResult>, String> {
        let row: Option<(
            uuid::Uuid,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
            i32,
            serde_json::Value,
            serde_json::Value,
        )> = sqlx::query_as(
            r#"
            SELECT id, target_url, status, start_time, end_time, total_requests, summary, errors
            FROM active_scans WHERE id = $1
            "#,
        )
        .bind(scan_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching active scan: {}", e))?;

        match row {
            Some(r) => {
                let status_val = serde_json::Value::String(r.2);
                let status: ActiveScanStatus =
                    serde_json::from_value(status_val).unwrap_or(ActiveScanStatus::Pending);
                let summary = serde_json::from_value(r.6).unwrap_or_default();
                let errors = serde_json::from_value(r.7).unwrap_or_default();

                Ok(Some(ActiveScanResult {
                    scan_id: r.0,
                    target_url: r.1,
                    status,
                    start_time: r.3,
                    end_time: r.4,
                    total_requests: r.5 as u32,
                    findings: vec![], // Findings are loaded separately if needed, or by join
                    summary,
                    errors,
                }))
            }
            None => Ok(None),
        }
    }

    async fn list_scans(
        &self,
        filters: &crate::domain::active_vuln::ScanQueryParams,
    ) -> Result<Vec<ActiveScanResult>, String> {
        let mut builder = sqlx::QueryBuilder::new(
            "SELECT id, target_url, status, start_time, end_time, total_requests, summary, errors FROM active_scans WHERE 1=1"
        );

        if let Some(ref target_url) = filters.target_url {
            builder.push(" AND target_url = ");
            builder.push_bind(target_url);
        }

        if let Some(ref status) = filters.status {
            let status_str = serde_json::to_value(status)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            builder.push(" AND status = ");
            builder.push_bind(status_str);
        }

        builder.push(" ORDER BY start_time DESC");

        let rows = builder
            .build_query_as::<(
                uuid::Uuid,
                String,
                String,
                chrono::DateTime<chrono::Utc>,
                Option<chrono::DateTime<chrono::Utc>>,
                i32,
                serde_json::Value,
                serde_json::Value,
            )>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error fetching active scans: {}", e))?;

        let mut scans = Vec::new();
        for r in rows {
            let status_val = serde_json::Value::String(r.2);
            let status: ActiveScanStatus =
                serde_json::from_value(status_val).unwrap_or(ActiveScanStatus::Pending);
            let summary = serde_json::from_value(r.6).unwrap_or_default();
            let errors = serde_json::from_value(r.7).unwrap_or_default();

            scans.push(ActiveScanResult {
                scan_id: r.0,
                target_url: r.1,
                status,
                start_time: r.3,
                end_time: r.4,
                total_requests: r.5 as u32,
                findings: vec![],
                summary,
                errors,
            });
        }
        Ok(scans)
    }

    async fn delete_scan(&self, scan_id: uuid::Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM active_scans WHERE id = $1")
            .bind(scan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error deleting active scan: {}", e))?;
        Ok(())
    }

    async fn update_scan(&self, scan: ActiveScanResult) -> Result<(), String> {
        let summary_json = serde_json::to_value(&scan.summary)
            .map_err(|e| format!("Failed to serialize summary: {}", e))?;
        let errors_json = serde_json::to_value(&scan.errors)
            .map_err(|e| format!("Failed to serialize errors: {}", e))?;
        let status_str = serde_json::to_value(&scan.status)
            .map_err(|e| format!("Failed to serialize status: {}", e))?
            .as_str()
            .unwrap_or("pending")
            .to_string();

        sqlx::query(
            r#"
            UPDATE active_scans
            SET target_url = $1, status = $2, start_time = $3, end_time = $4,
                total_requests = $5, summary = $6, errors = $7
            WHERE id = $8
            "#,
        )
        .bind(&scan.target_url)
        .bind(status_str)
        .bind(scan.start_time)
        .bind(scan.end_time)
        .bind(scan.total_requests as i32)
        .bind(&summary_json)
        .bind(&errors_json)
        .bind(scan.scan_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error updating active scan: {}", e))?;

        Ok(())
    }
}
