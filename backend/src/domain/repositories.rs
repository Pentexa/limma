use crate::domain::entities::WebScanResult;
use async_trait::async_trait;

#[async_trait]
pub trait WebsiteScanner: Send + Sync {
    async fn scan(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<WebScanResult, String>;

    async fn scan_stream(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::ScanEvent>,
    ) -> Result<WebScanResult, String>;
}

#[async_trait]
pub trait ServerInvestigator: Send + Sync {
    async fn investigate(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::ServerInfo, String>;

    async fn investigate_stream(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::InvestigationEvent>,
    ) -> Result<crate::domain::entities::ServerInfo, String>;
}

#[async_trait]
pub trait ApiDiscoverer: Send + Sync {
    async fn discover(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::ApiDiscoveryResult, String>;
}

#[async_trait]
pub trait ServiceCollector: Send + Sync {
    async fn collect(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::CollectorSnapshot, String>;
}

#[async_trait]
pub trait SecurityAuditorRepository: Send + Sync {
    async fn audit(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::SecurityReport, String>;

    async fn normalize_all(
        &self,
        target: &str,
        web_scan: &WebScanResult,
        server_info: &crate::domain::entities::ServerInfo,
        api_discovery: &crate::domain::entities::ApiDiscoveryResult,
        dynamic_engine: Option<&crate::infrastructure::rule_engine::DynamicRuleEngine>,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::NormalizedAuditReport, String>;
}

#[async_trait]
pub trait FormMapperRepository: Send + Sync {
    async fn map(
        &self,
        url: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<crate::domain::entities::FormMapping, String>;
}

// ── Faz F: Blind Detection & Exploitation Repositories ──

#[async_trait]

pub trait BlindFindingRepository: Send + Sync {
    async fn save(&self, finding: &crate::domain::entities::BlindFinding) -> Result<(), String>;
    async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<crate::domain::entities::BlindFinding>, String>;
    async fn find_by_scan(
        &self,
        scan_id: uuid::Uuid,
    ) -> Result<Vec<crate::domain::entities::BlindFinding>, String>;
    async fn update_verification(&self, id: uuid::Uuid, verified: bool) -> Result<(), String>;
}

#[async_trait]

pub trait PocRepository: Send + Sync {
    async fn save(&self, poc: &crate::domain::entities::Poc) -> Result<(), String>;
    async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<crate::domain::entities::Poc>, String>;
    async fn find_by_finding(
        &self,
        finding_id: uuid::Uuid,
    ) -> Result<Vec<crate::domain::entities::Poc>, String>;
    async fn update_verification(
        &self,
        id: uuid::Uuid,
        status: crate::domain::entities::ExploitVerificationStatus,
    ) -> Result<(), String>;
}

#[async_trait]

pub trait ExploitResultRepository: Send + Sync {
    async fn save(&self, result: &crate::domain::entities::ExploitResult) -> Result<(), String>;
    async fn find_by_poc(
        &self,
        poc_id: uuid::Uuid,
    ) -> Result<Vec<crate::domain::entities::ExploitResult>, String>;
}

// ── Phase 4: System Settings Repository ──

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_all_profiles(
        &self,
    ) -> Result<Vec<crate::domain::entities::SettingsProfile>, String>;
    async fn get_profile(
        &self,
        id: &str,
    ) -> Result<Option<crate::domain::entities::SettingsProfile>, String>;
    async fn save_profile(
        &self,
        profile: crate::domain::entities::SettingsProfile,
    ) -> Result<(), String>;
    async fn init_defaults(&self) -> Result<(), String>;
}

// ── Active Vulnerability Detection Phase ──

#[async_trait]
pub trait ActiveScanRepository: Send + Sync {
    async fn create_scan(
        &self,
        scan: crate::domain::active_vuln::ActiveScanResult,
    ) -> Result<(), String>;
    async fn update_status(
        &self,
        scan_id: uuid::Uuid,
        status: crate::domain::active_vuln::ActiveScanStatus,
    ) -> Result<(), String>;
    async fn find_by_id(
        &self,
        scan_id: uuid::Uuid,
    ) -> Result<Option<crate::domain::active_vuln::ActiveScanResult>, String>;
    async fn list_scans(
        &self,
        filters: &crate::domain::active_vuln::ScanQueryParams,
    ) -> Result<Vec<crate::domain::active_vuln::ActiveScanResult>, String>;
    async fn delete_scan(&self, scan_id: uuid::Uuid) -> Result<(), String>;
    
    async fn update_scan(
        &self,
        scan: crate::domain::active_vuln::ActiveScanResult,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait ActiveFindingRepository: Send + Sync {
    async fn save_finding(
        &self,
        finding: crate::domain::active_vuln::ActiveVulnFinding,
    ) -> Result<(), String>;
    async fn find_by_id(
        &self,
        finding_id: uuid::Uuid,
    ) -> Result<Option<crate::domain::active_vuln::ActiveVulnFinding>, String>;
    async fn find_by_scan_id(
        &self,
        scan_id: uuid::Uuid,
    ) -> Result<Vec<crate::domain::active_vuln::ActiveVulnFinding>, String>;
    async fn update_poc_id(&self, finding_id: uuid::Uuid, poc_id: uuid::Uuid)
        -> Result<(), String>;
    async fn find_by_filters(
        &self,
        params: &crate::domain::active_vuln::ActiveFindingQueryParams,
    ) -> Result<Vec<crate::domain::active_vuln::ActiveVulnFinding>, String>;
    async fn update_status(
        &self,
        finding_id: uuid::Uuid,
        verified: bool,
        false_positive: bool,
    ) -> Result<(), String>;
    async fn delete_by_scan_id(&self, scan_id: uuid::Uuid) -> Result<u64, String>;
}
