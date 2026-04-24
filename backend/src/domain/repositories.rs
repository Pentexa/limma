use crate::domain::entities::{User, WebScanResult};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: User) -> Result<(), String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
}

#[async_trait]
pub trait WebsiteScanner: Send + Sync {
    async fn scan(&self, url: &str) -> Result<WebScanResult, String>;
    async fn scan_stream(
        &self,
        url: &str,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::ScanEvent>,
    ) -> Result<WebScanResult, String>;
}

#[async_trait]
pub trait ServerInvestigator: Send + Sync {
    async fn investigate(&self, url: &str) -> Result<crate::domain::entities::ServerInfo, String>;
    async fn investigate_stream(
        &self,
        url: &str,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::InvestigationEvent>,
    ) -> Result<crate::domain::entities::ServerInfo, String>;
}

#[async_trait]
pub trait ApiDiscoverer: Send + Sync {
    async fn discover(
        &self,
        url: &str,
    ) -> Result<crate::domain::entities::ApiDiscoveryResult, String>;
}

#[async_trait]
pub trait ServiceCollector: Send + Sync {
    async fn collect(
        &self,
        url: &str,
    ) -> Result<crate::domain::entities::CollectorSnapshot, String>;
}

#[async_trait]
pub trait SecurityAuditorRepository: Send + Sync {
    async fn audit(&self, url: &str) -> Result<crate::domain::entities::SecurityReport, String>;

    async fn normalize_all(
        &self,
        target: &str,
        web_scan: &WebScanResult,
        server_info: &crate::domain::entities::ServerInfo,
        api_discovery: &crate::domain::entities::ApiDiscoveryResult,
        dynamic_engine: Option<&crate::infrastructure::rule_engine::DynamicRuleEngine>,
    ) -> Result<crate::domain::entities::NormalizedAuditReport, String>;
}

#[async_trait]
pub trait FormMapperRepository: Send + Sync {
    async fn map(&self, url: &str) -> Result<crate::domain::entities::FormMapping, String>;
}
