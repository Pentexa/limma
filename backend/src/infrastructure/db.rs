use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    ConnectOptions, PgPool,
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

    // Create required tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration: add password_hash column if the table already existed without it
    sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255) NOT NULL DEFAULT '';
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS learning_feedback (
            id SERIAL PRIMARY KEY,
            signature VARCHAR(512) NOT NULL,
            action VARCHAR(50) NOT NULL,
            timestamp_sec BIGINT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS confidence_calibration (
            signature VARCHAR(512) PRIMARY KEY,
            total_observations INTEGER NOT NULL DEFAULT 0,
            successful_verifications INTEGER NOT NULL DEFAULT 0,
            failed_verifications INTEGER NOT NULL DEFAULT 0,
            partial_verifications INTEGER NOT NULL DEFAULT 0,
            average_reproducibility REAL NOT NULL DEFAULT 0.0
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Delta Engine & Historical Data Tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scan_sessions (
            id UUID PRIMARY KEY,
            target_url VARCHAR(512) NOT NULL,
            timestamp_sec BIGINT NOT NULL,
            score REAL NOT NULL,
            total_endpoints INTEGER NOT NULL,
            total_findings INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scan_endpoints (
            id SERIAL PRIMARY KEY,
            scan_id UUID NOT NULL REFERENCES scan_sessions(id) ON DELETE CASCADE,
            url VARCHAR(2048) NOT NULL,
            method VARCHAR(20) NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_scan_endpoints_scan_id ON scan_endpoints(scan_id);",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scan_findings (
            id SERIAL PRIMARY KEY,
            scan_id UUID NOT NULL REFERENCES scan_sessions(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            severity VARCHAR(50) NOT NULL,
            url VARCHAR(2048) NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'Open'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scan_findings_scan_id ON scan_findings(scan_id);")
        .execute(&pool)
        .await?;

    Ok(pool)
}
