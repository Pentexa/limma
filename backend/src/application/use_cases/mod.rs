pub mod active_scan;
pub mod blind_scan;
pub mod generate_poc;
pub mod verify_exploit;

// ── Re-exported original use cases (migrated from use_cases.rs) ──

pub struct AnalyzeWebsite<'a, S: crate::domain::repositories::WebsiteScanner> {
    pub scanner: &'a S,
}

impl<'a, S: crate::domain::repositories::WebsiteScanner> AnalyzeWebsite<'a, S> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::WebScanResult, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.scanner.scan(&url, &config).await
    }
}

pub struct InvestigateServer<'a, I: crate::domain::repositories::ServerInvestigator> {
    pub investigator: &'a I,
}

impl<'a, I: crate::domain::repositories::ServerInvestigator> InvestigateServer<'a, I> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::ServerInfo, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.investigator.investigate(&url, &config).await
    }
}

pub struct DiscoverApis<'a, D: crate::domain::repositories::ApiDiscoverer> {
    pub discoverer: &'a D,
}

impl<'a, D: crate::domain::repositories::ApiDiscoverer> DiscoverApis<'a, D> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::ApiDiscoveryResult, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.discoverer.discover(&url, &config).await
    }
}

pub struct CollectExternalServices<'a, C: crate::domain::repositories::ServiceCollector> {
    pub collector: &'a C,
}

impl<'a, C: crate::domain::repositories::ServiceCollector> CollectExternalServices<'a, C> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::CollectorSnapshot, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.collector.collect(&url, &config).await
    }
}

pub struct AuditSecurity<'a, A: crate::domain::repositories::SecurityAuditorRepository> {
    pub auditor: &'a A,
}

impl<'a, A: crate::domain::repositories::SecurityAuditorRepository> AuditSecurity<'a, A> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::SecurityReport, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.auditor.audit(&url, &config).await
    }
}

pub struct MapForms<'a, M: crate::domain::repositories::FormMapperRepository> {
    pub mapper: &'a M,
}

impl<'a, M: crate::domain::repositories::FormMapperRepository> MapForms<'a, M> {
    pub async fn execute(
        &self,
        url: String,
        profile: &crate::domain::entities::SettingsProfile,
    ) -> Result<crate::domain::entities::FormMapping, String> {
        let config = crate::domain::engine_config::EngineConfig::from_profile(profile);
        self.mapper.map(&url, &config).await
    }
}

pub struct GenerateMasterReport<
    'a,
    S: crate::domain::repositories::WebsiteScanner,
    I: crate::domain::repositories::ServerInvestigator,
    D: crate::domain::repositories::ApiDiscoverer,
    C: crate::domain::repositories::ServiceCollector,
    A: crate::domain::repositories::SecurityAuditorRepository,
    M: crate::domain::repositories::FormMapperRepository,
> {
    pub scanner: &'a S,
    pub investigator: &'a I,
    pub discoverer: &'a D,
    pub collector: &'a C,
    pub auditor: &'a A,
    pub mapper: &'a M,
    pub dynamic_rule_engine: Option<&'a crate::infrastructure::rule_engine::DynamicRuleEngine>,
    pub db_pool: sqlx::PgPool,
    pub settings_repo: &'a dyn crate::domain::repositories::SettingsRepository,
}

impl<
        'a,
        S: crate::domain::repositories::WebsiteScanner,
        I: crate::domain::repositories::ServerInvestigator,
        D: crate::domain::repositories::ApiDiscoverer,
        C: crate::domain::repositories::ServiceCollector,
        A: crate::domain::repositories::SecurityAuditorRepository,
        M: crate::domain::repositories::FormMapperRepository,
    > GenerateMasterReport<'a, S, I, D, C, A, M>
{
    pub async fn execute(
        &self,
        url: String,
        profile_id: Option<String>,
    ) -> Result<crate::domain::entities::MasterReport, String> {
        // Resolve profile: use provided profile_id or fallback to "default"
        let profile_key = profile_id.unwrap_or_else(|| "default".to_string());
        let profile = self
            .settings_repo
            .get_profile(&profile_key)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(crate::domain::entities::SettingsProfile::default);

        let mut module_errors: Vec<String> = Vec::new();

        let config = crate::domain::engine_config::EngineConfig::from_profile(&profile);

        // --- PHASE 1: Reconnaissance (all parallel, individually failable) ---
        let (analysis_res, server_info_res, form_mapping_res, api_discovery_res) = tokio::join!(
            self.scanner.scan(&url, &config),
            self.investigator.investigate(&url, &config),
            self.mapper.map(&url, &config),
            self.discoverer.discover(&url, &config)
        );

        let analysis = match analysis_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[WebScanner] {}", e));
                None
            }
        };
        let server_info = match server_info_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[ServerInvestigator] {}", e));
                None
            }
        };
        let form_mapping = match form_mapping_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[FormMapper] {}", e));
                None
            }
        };
        let api_discovery = match api_discovery_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[ApiDiscoverer] {}", e));
                None
            }
        };

        // --- PHASE 2: Autonomous Scan Strategy ---
        let scan_strategy = if let (Some(ref a), Some(ref api), Some(ref fm)) =
            (&analysis, &api_discovery, &form_mapping)
        {
            let strategy_engine =
                crate::application::scan_strategy::AutonomousScanStrategyEngine::new(
                    self.db_pool.clone(),
                );
            Some(strategy_engine.compute_strategy(a, api, fm).await)
        } else {
            None
        };

        // --- PHASE 3: Deep Scan Execution (parallel, individually failable) ---
        let (service_collector_res, security_audit_res) = tokio::join!(
            self.collector.collect(&url, &config),
            self.auditor.audit(&url, &config)
        );

        let service_collector = match service_collector_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[ServiceCollector] {}", e));
                None
            }
        };
        let security_audit = match security_audit_res {
            Ok(v) => Some(v),
            Err(e) => {
                module_errors.push(format!("[SecurityAuditor] {}", e));
                None
            }
        };

        // --- PHASE 4: Normalized Audit (requires scan data) ---
        let normalized_audit = if let (Some(ref a), Some(ref si), Some(ref api)) =
            (&analysis, &server_info, &api_discovery)
        {
            self.auditor
                .normalize_all(&url, a, si, api, self.dynamic_rule_engine, &config)
                .await
                .ok()
        } else {
            module_errors
                .push("[NormalizedAudit] Skipped — prerequisite modules failed".to_string());
            None
        };

        let overall_health_score = security_audit
            .as_ref()
            .map(|sa| sa.security_score)
            .unwrap_or(0);

        Ok(crate::domain::entities::MasterReport {
            url,
            analysis,
            server_info,
            api_discovery,
            service_collector,
            security_audit,
            normalized_audit,
            form_mapping,
            scan_strategy,
            overall_health_score,
            module_errors,
        })
    }
}
