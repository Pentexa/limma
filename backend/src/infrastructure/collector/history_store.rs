use crate::domain::entities::CollectorSnapshot;
use sqlx::PgPool;

/// PostgreSQL-backed collector history. Keeping this as a small value object
/// makes persistence explicit and avoids process-local state.
#[derive(Clone)]
pub struct HistoryStore {
    pool: PgPool,
}

impl HistoryStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_snapshot(
        &self,
        target: &str,
        snapshot: &CollectorSnapshot,
    ) -> Result<(), String> {
        let payload = serde_json::to_value(snapshot)
            .map_err(|e| format!("Failed to serialize collector snapshot: {e}"))?;

        sqlx::query(
            "INSERT INTO collector_snapshots (target_url, captured_at, snapshot) VALUES ($1, $2, $3)",
        )
        .bind(target)
        .bind(snapshot.timestamp)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to persist collector snapshot: {e}"))?;
        Ok(())
    }

    pub async fn get_previous_snapshot(
        &self,
        target: &str,
    ) -> Result<Option<CollectorSnapshot>, String> {
        let payload = sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT snapshot FROM collector_snapshots WHERE target_url = $1 ORDER BY captured_at DESC, id DESC LIMIT 1",
        )
        .bind(target)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load collector history: {e}"))?;

        payload
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| format!("Stored collector snapshot is invalid: {e}"))
    }
}
