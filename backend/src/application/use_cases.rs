use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use uuid::Uuid;
use chrono::Utc;

pub struct RegisterUser<'a, R: UserRepository> {
    pub repo: &'a R,
}

impl<'a, R: UserRepository> RegisterUser<'a, R> {
    pub async fn execute(&self, name: String, email: String) -> Result<User, String> {
        let existing = self.repo.find_by_email(&email).await?;
        if existing.is_some() {
            return Err("User already exists".to_string());
        }

        let id = Uuid::new_v4();
        let user = User {
            id,
            name,
            email,
            created_at: Utc::now(),
        };

        self.repo.save(user.clone()).await?;

        Ok(user)
    }
}

pub struct AnalyzeWebsite<'a, S: crate::domain::repositories::WebsiteScanner> {
    pub scanner: &'a S,
}

impl<'a, S: crate::domain::repositories::WebsiteScanner> AnalyzeWebsite<'a, S> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::WebScanResult, String> {
        self.scanner.scan(&url).await
    }
}

pub struct InvestigateServer<'a, I: crate::domain::repositories::ServerInvestigator> {
    pub investigator: &'a I,
}

impl<'a, I: crate::domain::repositories::ServerInvestigator> InvestigateServer<'a, I> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::ServerInfo, String> {
        self.investigator.investigate(&url).await
    }
}

pub struct DiscoverApis<'a, D: crate::domain::repositories::ApiDiscoverer> {
    pub discoverer: &'a D,
}

impl<'a, D: crate::domain::repositories::ApiDiscoverer> DiscoverApis<'a, D> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::ApiDiscoveryResult, String> {
        self.discoverer.discover(&url).await
    }
}

pub struct CollectExternalServices<'a, C: crate::domain::repositories::ServiceCollector> {
    pub collector: &'a C,
}

impl<'a, C: crate::domain::repositories::ServiceCollector> CollectExternalServices<'a, C> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::CollectorSnapshot, String> {
        self.collector.collect(&url).await
    }
}

pub struct AuditSecurity<'a, A: crate::domain::repositories::SecurityAuditorRepository> {
    pub auditor: &'a A,
}

impl<'a, A: crate::domain::repositories::SecurityAuditorRepository> AuditSecurity<'a, A> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::SecurityReport, String> {
        self.auditor.audit(&url).await
    }
}

pub struct MapForms<'a, M: crate::domain::repositories::FormMapperRepository> {
    pub mapper: &'a M,
}

impl<'a, M: crate::domain::repositories::FormMapperRepository> MapForms<'a, M> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::FormMapping, String> {
        self.mapper.map(&url).await
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
}

impl<
    'a,
    S: crate::domain::repositories::WebsiteScanner,
    I: crate::domain::repositories::ServerInvestigator,
    D: crate::domain::repositories::ApiDiscoverer,
    C: crate::domain::repositories::ServiceCollector,
    A: crate::domain::repositories::SecurityAuditorRepository,
    M: crate::domain::repositories::FormMapperRepository,
> GenerateMasterReport<'a, S, I, D, C, A, M> {
    pub async fn execute(&self, url: String) -> Result<crate::domain::entities::MasterReport, String> {
        // --- PHASE 1: Reconnaissance ---
        let (analysis, server_info, form_mapping) = 
            tokio::try_join!(
                self.scanner.scan(&url),
                self.investigator.investigate(&url),
                self.mapper.map(&url)
            )?;

        // --- PHASE 2: Discovery (Needed for strategy) ---
        let api_discovery = self.discoverer.discover(&url).await?;

        // --- PHASE 3: Autonomous Scan Strategy ---
        let strategy_engine = crate::application::scan_strategy::AutonomousScanStrategyEngine::new();
        let scan_strategy = strategy_engine.compute_strategy(&analysis, &api_discovery, &form_mapping);

        // --- PHASE 4: Deep Scan Execution ---
        let (service_collector, security_audit) = 
            tokio::try_join!(
                self.collector.collect(&url),
                self.auditor.audit(&url)
            )?;

        let normalized_audit = self.auditor.normalize_all(&url, &analysis, &server_info, &api_discovery).await.ok();

        let overall_health_score = security_audit.security_score; // Basic heuristic

        Ok(crate::domain::entities::MasterReport {
            url,
            analysis,
            server_info,
            api_discovery,
            service_collector,
            security_audit,
            normalized_audit,
            form_mapping,
            scan_strategy: Some(scan_strategy),
            overall_health_score,
        })
    }
}







