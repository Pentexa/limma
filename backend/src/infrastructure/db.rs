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

    // ── Burp Bridge Persistence Tables ──

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS burp_sessions (
            session_id VARCHAR(128) PRIMARY KEY,
            target_url VARCHAR(2048) NOT NULL,
            burp_version VARCHAR(128),
            plugin_version VARCHAR(128),
            connected_at TIMESTAMPTZ NOT NULL,
            last_heartbeat TIMESTAMPTZ NOT NULL,
            imported_traffic_count INTEGER NOT NULL DEFAULT 0,
            exported_findings_count INTEGER NOT NULL DEFAULT 0,
            status VARCHAR(50) NOT NULL DEFAULT 'connected'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS burp_traffic_items (
            id SERIAL PRIMARY KEY,
            session_id VARCHAR(128) NOT NULL REFERENCES burp_sessions(session_id) ON DELETE CASCADE,
            url VARCHAR(2048) NOT NULL,
            method VARCHAR(20) NOT NULL,
            request_headers JSONB NOT NULL DEFAULT '{}',
            request_body TEXT,
            response_status INTEGER NOT NULL,
            response_headers JSONB NOT NULL DEFAULT '{}',
            response_body TEXT,
            timestamp BIGINT NOT NULL,
            tool_source VARCHAR(50) NOT NULL DEFAULT 'proxy'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_burp_traffic_session ON burp_traffic_items(session_id);",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS burp_findings (
            id SERIAL PRIMARY KEY,
            session_id VARCHAR(128) NOT NULL REFERENCES burp_sessions(session_id) ON DELETE CASCADE,
            name VARCHAR(512) NOT NULL,
            detail TEXT NOT NULL DEFAULT '',
            severity VARCHAR(50) NOT NULL,
            confidence VARCHAR(50) NOT NULL,
            url VARCHAR(2048) NOT NULL,
            path VARCHAR(2048) NOT NULL DEFAULT '/',
            host VARCHAR(512) NOT NULL,
            port INTEGER NOT NULL DEFAULT 443,
            protocol VARCHAR(20) NOT NULL DEFAULT 'https',
            remediation TEXT NOT NULL DEFAULT '',
            issue_type INTEGER NOT NULL DEFAULT 0,
            cwe_id INTEGER
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_burp_findings_session ON burp_findings(session_id);",
    )
    .execute(&pool)
    .await?;

    // ── Faz F: Blind Detection & Active Exploitation Tables ──

    // Drop the foreign key constraint if it exists (ignoring errors if it doesn't)
    let _ = sqlx::query(
        "ALTER TABLE blind_findings DROP CONSTRAINT IF EXISTS blind_findings_scan_id_fkey;",
    )
    .execute(&pool)
    .await;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blind_findings (
            id UUID PRIMARY KEY,
            scan_id UUID NOT NULL,
            target_id UUID NOT NULL,
            vulnerability_type VARCHAR(100) NOT NULL,
            detection_method VARCHAR(100) NOT NULL,
            confidence REAL NOT NULL,
            evidence JSONB NOT NULL DEFAULT '{}',
            payload_used TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            verified BOOLEAN NOT NULL DEFAULT false
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_blind_findings_scan_id ON blind_findings(scan_id);",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pocs (
            id UUID PRIMARY KEY,
            finding_id UUID NOT NULL,
            poc_type VARCHAR(100) NOT NULL,
            code TEXT NOT NULL,
            language VARCHAR(50) NOT NULL,
            safety_level VARCHAR(50) NOT NULL,
            verification_status VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_pocs_finding_id ON pocs(finding_id);")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exploit_results (
            id UUID PRIMARY KEY,
            poc_id UUID NOT NULL REFERENCES pocs(id) ON DELETE CASCADE,
            executed_at TIMESTAMPTZ NOT NULL,
            success BOOLEAN NOT NULL,
            output TEXT,
            error TEXT,
            execution_time_ms BIGINT NOT NULL,
            sandbox_logs TEXT
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_exploit_results_poc_id ON exploit_results(poc_id);",
    )
    .execute(&pool)
    .await?;

    // ── Phase 4: Settings Profiles Table ──

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings_profiles (
            id VARCHAR(128) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            is_custom BOOLEAN NOT NULL DEFAULT false,
            profile_data JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Active Detection Tables ──

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS active_scans (
            id UUID PRIMARY KEY,
            target_url VARCHAR(2048) NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'pending',
            start_time TIMESTAMPTZ NOT NULL,
            end_time TIMESTAMPTZ,
            total_requests INTEGER NOT NULL DEFAULT 0,
            summary JSONB NOT NULL DEFAULT '{}',
            errors JSONB NOT NULL DEFAULT '[]'
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS active_findings (
            id UUID PRIMARY KEY,
            scan_id UUID NOT NULL REFERENCES active_scans(id) ON DELETE CASCADE,
            timestamp TIMESTAMPTZ NOT NULL,
            vuln_type VARCHAR(100) NOT NULL,
            target_url VARCHAR(2048) NOT NULL,
            affected_parameter VARCHAR(255) NOT NULL,
            http_method VARCHAR(20) NOT NULL,
            payload_used TEXT NOT NULL,
            evidence JSONB NOT NULL,
            severity VARCHAR(50) NOT NULL,
            confidence VARCHAR(50) NOT NULL,
            exploitability VARCHAR(50) NOT NULL,
            poc_generated BOOLEAN NOT NULL DEFAULT false,
            poc_id UUID,
            verified BOOLEAN NOT NULL DEFAULT false,
            false_positive BOOLEAN NOT NULL DEFAULT false
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_active_findings_scan_id ON active_findings(scan_id);",
    )
    .execute(&pool)
    .await?;

    // Phase 4: Missing Tables and Triggers
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payload_library (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            vuln_type VARCHAR(100) NOT NULL,
            payload TEXT NOT NULL,
            description TEXT,
            expected_indicator_type VARCHAR(50) NOT NULL,
            expected_indicator_value TEXT,
            severity VARCHAR(20) NOT NULL,
            safe_for_production BOOLEAN DEFAULT TRUE,
            context_requirements JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_payload_library_vuln_type ON payload_library(vuln_type);",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exploit_verifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            finding_id UUID NOT NULL REFERENCES active_findings(id) ON DELETE CASCADE,
            poc_id UUID REFERENCES pocs(id),
            execution_level VARCHAR(20) NOT NULL,
            sandbox_type VARCHAR(50),
            success BOOLEAN NOT NULL,
            output TEXT,
            error TEXT,
            execution_time_ms INTEGER,
            artifacts JSONB,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            executed_by UUID
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_active_scan_stats()
        RETURNS TRIGGER AS $$
        BEGIN
            UPDATE active_scans
            SET 
                status = CASE 
                    WHEN NEW.timestamp > NOW() - INTERVAL '5 minutes' THEN 'running'
                    ELSE 'completed'
                END,
                end_time = CASE 
                    WHEN (SELECT COUNT(*) FROM active_findings WHERE scan_id = NEW.scan_id) > 50 
                    THEN NOW()
                    ELSE NULL
                END
            WHERE id = NEW.scan_id;
            
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("DROP TRIGGER IF EXISTS trigger_update_scan_stats ON active_findings;")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER trigger_update_scan_stats
            AFTER INSERT ON active_findings
            FOR EACH ROW
            EXECUTE FUNCTION update_active_scan_stats();
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Custom Rules Table ──

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS custom_rules (
            id VARCHAR(128) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            yaml_content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
