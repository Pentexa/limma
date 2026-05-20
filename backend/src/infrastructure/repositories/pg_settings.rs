use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::SettingsProfile;
use crate::domain::repositories::SettingsRepository;

/// PostgreSQL-backed settings repository.
/// Stores each profile as a row with the full profile serialized into JSONB.
/// On startup, seeds default profiles if the table is empty.
pub struct PgSettingsRepository {
    pool: PgPool,
}

impl PgSettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Build the same default profiles that InMemorySettingsRepository uses.
    fn build_default_profiles() -> Vec<SettingsProfile> {
        use crate::domain::entities::*;

        let base = SettingsProfile {
            id: "default".to_string(),
            name: "Default Profile".to_string(),
            description: "Standard balanced scan settings".to_string(),
            is_custom: false,
            global: GlobalSettings {
                timeout_ms: 10000,
                rate_limit_req_per_sec: 50,
                use_proxy: false,
                proxy_url: "".to_string(),
                target_scope: "strict".to_string(),
                auth_profile_id: None,
            },
            scanner: ScannerSettings {
                user_agent: "Limma/1.0".to_string(),
                wordlist_size: "medium".to_string(),
                follow_redirects: true,
                max_depth: 3,
            },
            investigator: InvestigatorSettings {
                dns_resolution: "fast".to_string(),
                fingerprint_level: 2,
                concurrent_hosts: 10,
            },
            api_discovery: ApiDiscoverySettings {
                wordlist_size: "medium".to_string(),
                custom_headers: false,
                schema_parsing: true,
            },
            services: ServiceCollectorSettings {
                port_scan_range: "top1000".to_string(),
                banner_grabbing: true,
                timeout_per_port_ms: 2000,
            },
            forms: FormMapperSettings {
                fuzzing_intensity: "medium".to_string(),
                extract_hidden_inputs: true,
                avoid_waf: true,
            },
            audit: AuditSettings {
                risk_coefficient: 1.0,
                ignore_informational: false,
                auto_map_cwe: true,
            },
            rules: RuleEngineSettings {
                strict_mode: false,
                auto_sync_rules: true,
                custom_rule_path: "".to_string(),
            },
            exploit: ExploitSettings {
                mode: "safe_verification".to_string(),
                sandbox_validation: true,
                manual_approval_required: true,
            },
            proxy: ProxySettings {
                intercept_requests: false,
                history_limit: 1000,
                auto_drop_malicious: false,
            },
            sessions: SessionSettings {
                auto_delete_days: 30,
                archive_artifacts: true,
            },
        };

        let mut profiles = vec![base.clone()];

        // Fast Profile
        let mut fast = base.clone();
        fast.id = "fast".to_string();
        fast.name = "Fast Scan".to_string();
        fast.description = "Quick surface-level reconnaissance".to_string();
        fast.global.timeout_ms = 5000;
        fast.global.rate_limit_req_per_sec = 200;
        fast.scanner.max_depth = 1;
        fast.scanner.wordlist_size = "small".to_string();
        fast.services.port_scan_range = "top100".to_string();
        fast.audit.risk_coefficient = 0.8;
        profiles.push(fast);

        // Deep Profile
        let mut deep = base.clone();
        deep.id = "deep".to_string();
        deep.name = "Deep Scan".to_string();
        deep.description = "Thorough analysis with extended timeouts".to_string();
        deep.global.timeout_ms = 60000;
        deep.global.rate_limit_req_per_sec = 20;
        deep.scanner.max_depth = 8;
        deep.scanner.wordlist_size = "large".to_string();
        deep.services.port_scan_range = "full".to_string();
        deep.services.timeout_per_port_ms = 5000;
        deep.audit.risk_coefficient = 1.2;
        profiles.push(deep);

        // Safe Profile
        let mut safe = base.clone();
        safe.id = "safe".to_string();
        safe.name = "Safe / Passive".to_string();
        safe.description = "Non-intrusive passive-only scanning".to_string();
        safe.global.rate_limit_req_per_sec = 5;
        safe.scanner.max_depth = 2;
        safe.scanner.wordlist_size = "small".to_string();
        safe.forms.fuzzing_intensity = "none".to_string();
        safe.forms.avoid_waf = true;
        safe.exploit.mode = "safe_verification".to_string();
        safe.exploit.sandbox_validation = true;
        safe.exploit.manual_approval_required = true;
        profiles.push(safe);

        // Red Team Profile
        let mut redteam = base;
        redteam.id = "redteam".to_string();
        redteam.name = "Red Team (Active)".to_string();
        redteam.description = "Aggressive offensive security testing with proxy support".to_string();
        redteam.global.timeout_ms = 20000;
        redteam.global.rate_limit_req_per_sec = 100;
        redteam.global.use_proxy = true;
        redteam.global.proxy_url = "http://127.0.0.1:8080".to_string();
        redteam.global.target_scope = "wildcard".to_string();
        redteam.scanner.max_depth = 5;
        redteam.scanner.wordlist_size = "large".to_string();
        redteam.services.port_scan_range = "full".to_string();
        redteam.forms.fuzzing_intensity = "high".to_string();
        redteam.audit.risk_coefficient = 1.5;
        redteam.exploit.mode = "authorized_active".to_string();
        redteam.exploit.sandbox_validation = false;
        redteam.exploit.manual_approval_required = false;
        profiles.push(redteam);

        profiles
    }
}

#[async_trait]
impl SettingsRepository for PgSettingsRepository {
    async fn get_all_profiles(&self) -> Result<Vec<SettingsProfile>, String> {
        let rows: Vec<(serde_json::Value,)> = sqlx::query_as(
            "SELECT profile_data FROM settings_profiles ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching profiles: {}", e))?;

        let profiles: Vec<SettingsProfile> = rows
            .into_iter()
            .filter_map(|(data,)| serde_json::from_value(data).ok())
            .collect();

        Ok(profiles)
    }

    async fn get_profile(&self, id: &str) -> Result<Option<SettingsProfile>, String> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT profile_data FROM settings_profiles WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching profile {}: {}", id, e))?;

        match row {
            Some((data,)) => {
                let profile: SettingsProfile = serde_json::from_value(data)
                    .map_err(|e| format!("Failed to deserialize profile {}: {}", id, e))?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    async fn save_profile(&self, profile: SettingsProfile) -> Result<(), String> {
        let profile_data = serde_json::to_value(&profile)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;

        sqlx::query(
            r#"
            INSERT INTO settings_profiles (id, name, description, is_custom, profile_data, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                is_custom = EXCLUDED.is_custom,
                profile_data = EXCLUDED.profile_data,
                updated_at = NOW()
            "#,
        )
        .bind(&profile.id)
        .bind(&profile.name)
        .bind(&profile.description)
        .bind(profile.is_custom)
        .bind(&profile_data)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error saving profile {}: {}", profile.id, e))?;

        Ok(())
    }

    async fn init_defaults(&self) -> Result<(), String> {
        // Check if any profiles exist
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM settings_profiles"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB error checking profile count: {}", e))?;

        if count.0 == 0 {
            tracing::info!("[PgSettingsRepository] No profiles found, seeding {} defaults...", Self::build_default_profiles().len());
            let defaults = Self::build_default_profiles();
            for profile in defaults {
                self.save_profile(profile).await?;
            }
            tracing::info!("[PgSettingsRepository] Default profiles seeded successfully");
        } else {
            tracing::info!("[PgSettingsRepository] {} existing profiles found, skipping seed", count.0);
        }

        Ok(())
    }
}
