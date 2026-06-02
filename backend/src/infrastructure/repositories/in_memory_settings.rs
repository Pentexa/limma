use crate::domain::entities::{
    ApiDiscoverySettings, AuditSettings, ExploitSettings, FormMapperSettings, GlobalSettings,
    InvestigatorSettings, ProxySettings, RuleEngineSettings, ScannerSettings,
    ServiceCollectorSettings, SessionSettings, SettingsProfile,
};
use crate::domain::repositories::SettingsRepository;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemorySettingsRepository {
    store: Arc<RwLock<HashMap<String, SettingsProfile>>>,
}

impl Default for InMemorySettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySettingsRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn build_default_profiles() -> Vec<SettingsProfile> {
        let mut profiles = Vec::new();

        let base_global = GlobalSettings {
            timeout_ms: 10000,
            rate_limit_req_per_sec: 50,
            use_proxy: false,
            proxy_url: "".to_string(),
            target_scope: "strict".to_string(),
            auth_profile_id: None,
        };
        let base_scanner = ScannerSettings {
            user_agent: "Limma/1.0".to_string(),
            wordlist_size: "medium".to_string(),
            follow_redirects: true,
            max_depth: 3,
        };
        let base_investigator = InvestigatorSettings {
            dns_resolution: "fast".to_string(),
            fingerprint_level: 2,
            concurrent_hosts: 10,
        };
        let base_api_discovery = ApiDiscoverySettings {
            wordlist_size: "medium".to_string(),
            custom_headers: false,
            schema_parsing: true,
        };
        let base_services = ServiceCollectorSettings {
            port_scan_range: "top1000".to_string(),
            banner_grabbing: true,
            timeout_per_port_ms: 2000,
        };
        let base_forms = FormMapperSettings {
            fuzzing_intensity: "medium".to_string(),
            extract_hidden_inputs: true,
            avoid_waf: true,
        };
        let base_audit = AuditSettings {
            risk_coefficient: 1.0,
            ignore_informational: false,
            auto_map_cwe: true,
        };
        let base_rules = RuleEngineSettings {
            strict_mode: false,
            auto_sync_rules: true,
            custom_rule_path: "".to_string(),
        };
        let base_exploit = ExploitSettings {
            mode: "safe_verification".to_string(),
            sandbox_validation: true,
            manual_approval_required: true,
        };
        let base_proxy = ProxySettings {
            intercept_requests: false,
            history_limit: 1000,
            auto_drop_malicious: false,
        };
        let base_sessions = SessionSettings {
            auto_delete_days: 30,
            archive_artifacts: true,
        };

        // 1. Default Profile
        profiles.push(SettingsProfile {
            id: "default".to_string(),
            name: "Default Profile".to_string(),
            description: "Standard balanced scan settings".to_string(),
            is_custom: false,
            global: base_global.clone(),
            scanner: base_scanner.clone(),
            investigator: base_investigator.clone(),
            api_discovery: base_api_discovery.clone(),
            services: base_services.clone(),
            forms: base_forms.clone(),
            audit: base_audit.clone(),
            rules: base_rules.clone(),
            exploit: base_exploit.clone(),
            proxy: base_proxy.clone(),
            sessions: base_sessions.clone(),
            subdomain: crate::domain::entities::SubdomainDiscoverySettings::default(),
        });

        // 2. Fast Profile
        let mut fast = profiles[0].clone();
        fast.id = "fast".to_string();
        fast.name = "Fast Scan".to_string();
        fast.global.timeout_ms = 5000;
        fast.global.rate_limit_req_per_sec = 200;
        fast.scanner.max_depth = 1;
        fast.scanner.wordlist_size = "small".to_string();
        fast.services.port_scan_range = "top100".to_string();
        fast.audit.risk_coefficient = 0.8;
        profiles.push(fast);

        // 3. Red Team Profile
        let mut redteam = profiles[0].clone();
        redteam.id = "redteam".to_string();
        redteam.name = "Red Team (Active)".to_string();
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
impl SettingsRepository for InMemorySettingsRepository {
    async fn get_all_profiles(&self) -> Result<Vec<SettingsProfile>, String> {
        let store = self.store.read().await;
        Ok(store.values().cloned().collect())
    }

    async fn get_profile(&self, id: &str) -> Result<Option<SettingsProfile>, String> {
        let store = self.store.read().await;
        Ok(store.get(id).cloned())
    }

    async fn save_profile(&self, profile: SettingsProfile) -> Result<(), String> {
        let mut store = self.store.write().await;
        store.insert(profile.id.clone(), profile);
        Ok(())
    }

    async fn init_defaults(&self) -> Result<(), String> {
        let mut store = self.store.write().await;
        if store.is_empty() {
            let defaults = Self::build_default_profiles();
            for profile in defaults {
                store.insert(profile.id.clone(), profile);
            }
        }
        Ok(())
    }
}
