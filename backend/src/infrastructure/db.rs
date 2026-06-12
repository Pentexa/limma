use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    PgPool,
};
use std::str::FromStr;
use std::time::Duration;

pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let connect_options = PgConnectOptions::from_str(database_url)?.ssl_mode(PgSslMode::Disable);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(connect_options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    #[test]
    fn baseline_migration_does_not_recreate_active_scan_status_trigger() {
        let migration = include_str!("../../migrations/202606120001_initial_schema.sql");

        assert!(migration.contains("DROP TRIGGER IF EXISTS trigger_update_scan_stats"));
        assert!(!migration.contains("CREATE TRIGGER trigger_update_scan_stats"));
        assert!(!migration.contains("CREATE OR REPLACE FUNCTION update_active_scan_stats"));
    }
}
