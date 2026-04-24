use crate::domain::entities::MasterReport;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub scan_id: Uuid,
    pub timestamp_sec: i64,
    pub score: f32,
    pub total_endpoints: i32,
    pub total_findings: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaResult {
    pub base_scan_id: Uuid,
    pub compare_scan_id: Uuid,
    pub new_endpoints: Vec<DeltaEndpoint>,
    pub resolved_findings: Vec<DeltaFinding>,
    pub new_findings: Vec<DeltaFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEndpoint {
    pub url: String,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaFinding {
    pub name: String,
    pub severity: String,
    pub url: String,
}

pub struct DeltaEngine {
    pool: PgPool,
}

impl DeltaEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_scan(&self, report: &MasterReport) -> Result<Uuid, sqlx::Error> {
        let scan_id = Uuid::new_v4();
        let target_url = &report.url;
        let timestamp_sec = Utc::now().timestamp();
        let score = report.overall_health_score as f32;

        let total_endpoints = report
            .api_discovery
            .as_ref()
            .map(|d| d.detected_endpoints.len())
            .unwrap_or(0) as i32;
        let total_findings = report
            .normalized_audit
            .as_ref()
            .map(|a| a.findings.len())
            .unwrap_or(0) as i32;

        // Insert scan session
        sqlx::query(
            "INSERT INTO scan_sessions (id, target_url, timestamp_sec, score, total_endpoints, total_findings) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(scan_id)
        .bind(target_url)
        .bind(timestamp_sec)
        .bind(score)
        .bind(total_endpoints)
        .bind(total_findings)
        .execute(&self.pool)
        .await?;

        // Insert endpoints
        if let Some(discovery) = &report.api_discovery {
            for ep in &discovery.detected_endpoints {
                sqlx::query(
                    "INSERT INTO scan_endpoints (scan_id, url, method) VALUES ($1, $2, $3)",
                )
                .bind(scan_id)
                .bind(&ep.path)
                .bind(&ep.method_prediction)
                .execute(&self.pool)
                .await?;
            }
        }

        // Insert findings
        if let Some(audit) = &report.normalized_audit {
            for finding in &audit.findings {
                let severity = format!("{:?}", finding.severity); // Converts enum to string

                sqlx::query(
                    "INSERT INTO scan_findings (scan_id, name, severity, url, status) VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(scan_id)
                .bind(&finding.summary)
                .bind(severity)
                .bind(finding.affected_path_or_endpoint.as_deref().unwrap_or(target_url))
                .bind("Open")
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(scan_id)
    }

    pub async fn get_trends(&self, target_url: &str) -> Result<Vec<TrendPoint>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RawTrend {
            id: Uuid,
            timestamp_sec: i64,
            score: f32,
            total_endpoints: i32,
            total_findings: i32,
        }

        let records: Vec<RawTrend> = sqlx::query_as(
            r#"
            SELECT id, timestamp_sec, score, total_endpoints, total_findings 
            FROM scan_sessions 
            WHERE target_url = $1 
            ORDER BY timestamp_sec ASC
            LIMIT 50
            "#,
        )
        .bind(target_url)
        .fetch_all(&self.pool)
        .await?;

        let mut trends = Vec::new();
        for r in records {
            trends.push(TrendPoint {
                scan_id: r.id,
                timestamp_sec: r.timestamp_sec,
                score: r.score,
                total_endpoints: r.total_endpoints,
                total_findings: r.total_findings,
            });
        }

        Ok(trends)
    }

    pub async fn calculate_delta(
        &self,
        target_url: &str,
        current_scan_id: Uuid,
        previous_scan_id: Uuid,
    ) -> Result<DeltaResult, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct EpRow {
            url: String,
            method: String,
        }

        #[derive(sqlx::FromRow)]
        struct FdRow {
            name: String,
            severity: String,
            url: String,
        }

        // Find new endpoints (in current but not in previous)
        let new_eps: Vec<EpRow> = sqlx::query_as(
            r#"
            SELECT url, method FROM scan_endpoints WHERE scan_id = $1
            EXCEPT
            SELECT url, method FROM scan_endpoints WHERE scan_id = $2
            "#,
        )
        .bind(current_scan_id)
        .bind(previous_scan_id)
        .fetch_all(&self.pool)
        .await?;

        // Find resolved findings (in previous but not in current)
        let resolved_fds: Vec<FdRow> = sqlx::query_as(
            r#"
            SELECT name, severity, url FROM scan_findings WHERE scan_id = $2
            EXCEPT
            SELECT name, severity, url FROM scan_findings WHERE scan_id = $1
            "#,
        )
        .bind(current_scan_id)
        .bind(previous_scan_id)
        .fetch_all(&self.pool)
        .await?;

        // Find new findings (in current but not in previous)
        let new_fds: Vec<FdRow> = sqlx::query_as(
            r#"
            SELECT name, severity, url FROM scan_findings WHERE scan_id = $1
            EXCEPT
            SELECT name, severity, url FROM scan_findings WHERE scan_id = $2
            "#,
        )
        .bind(current_scan_id)
        .bind(previous_scan_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(DeltaResult {
            base_scan_id: previous_scan_id,
            compare_scan_id: current_scan_id,
            new_endpoints: new_eps
                .into_iter()
                .map(|r| DeltaEndpoint {
                    url: r.url,
                    method: r.method,
                })
                .collect(),
            resolved_findings: resolved_fds
                .into_iter()
                .map(|r| DeltaFinding {
                    name: r.name,
                    severity: r.severity,
                    url: r.url,
                })
                .collect(),
            new_findings: new_fds
                .into_iter()
                .map(|r| DeltaFinding {
                    name: r.name,
                    severity: r.severity,
                    url: r.url,
                })
                .collect(),
        })
    }
}
