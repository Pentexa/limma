use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use chrono::Utc;
use uuid::Uuid;

pub struct RegisterUser<'a, R: UserRepository> {
    pub repo: &'a R,
}

impl<'a, R: UserRepository> RegisterUser<'a, R> {
    pub async fn execute(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, String> {
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        let existing = self.repo.find_by_email(&email).await?;
        if existing.is_some() {
            return Err("User already exists".to_string());
        }

        let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Password hashing failed: {}", e))?;

        let id = Uuid::new_v4();
        let user = User {
            id,
            name,
            email,
            password_hash,
            created_at: Utc::now(),
        };

        self.repo.save(user.clone()).await?;

        Ok(user)
    }
}

pub struct LoginUser<'a, R: UserRepository> {
    pub repo: &'a R,
    pub jwt_secret: &'a str,
}

impl<'a, R: UserRepository> LoginUser<'a, R> {
    pub async fn execute(&self, email: String, password: String) -> Result<(User, String), String> {
        let user = self
            .repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| "Invalid email or password".to_string())?;

        let valid = bcrypt::verify(&password, &user.password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))?;

        if !valid {
            return Err("Invalid email or password".to_string());
        }

        let token = crate::infrastructure::auth::create_token(user.id, self.jwt_secret)?;

        Ok((user, token))
    }
}

pub struct AnalyzeWebsite<'a, S: crate::domain::repositories::WebsiteScanner> {
    pub scanner: &'a S,
}

impl<'a, S: crate::domain::repositories::WebsiteScanner> AnalyzeWebsite<'a, S> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::WebScanResult, String> {
        self.scanner.scan(&url).await
    }
}

pub struct InvestigateServer<'a, I: crate::domain::repositories::ServerInvestigator> {
    pub investigator: &'a I,
}

impl<'a, I: crate::domain::repositories::ServerInvestigator> InvestigateServer<'a, I> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::ServerInfo, String> {
        self.investigator.investigate(&url).await
    }
}

pub struct DiscoverApis<'a, D: crate::domain::repositories::ApiDiscoverer> {
    pub discoverer: &'a D,
}

impl<'a, D: crate::domain::repositories::ApiDiscoverer> DiscoverApis<'a, D> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::ApiDiscoveryResult, String> {
        self.discoverer.discover(&url).await
    }
}

pub struct CollectExternalServices<'a, C: crate::domain::repositories::ServiceCollector> {
    pub collector: &'a C,
}

impl<'a, C: crate::domain::repositories::ServiceCollector> CollectExternalServices<'a, C> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::CollectorSnapshot, String> {
        self.collector.collect(&url).await
    }
}

pub struct AuditSecurity<'a, A: crate::domain::repositories::SecurityAuditorRepository> {
    pub auditor: &'a A,
}

impl<'a, A: crate::domain::repositories::SecurityAuditorRepository> AuditSecurity<'a, A> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::SecurityReport, String> {
        self.auditor.audit(&url).await
    }
}

pub struct MapForms<'a, M: crate::domain::repositories::FormMapperRepository> {
    pub mapper: &'a M,
}

impl<'a, M: crate::domain::repositories::FormMapperRepository> MapForms<'a, M> {
    pub async fn execute(
        &self,
        url: String,
    ) -> Result<crate::domain::entities::FormMapping, String> {
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
    pub dynamic_rule_engine: Option<&'a crate::infrastructure::rule_engine::DynamicRuleEngine>,
    pub db_pool: sqlx::PgPool,
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
    ) -> Result<crate::domain::entities::MasterReport, String> {
        let mut module_errors: Vec<String> = Vec::new();

        // --- PHASE 1: Reconnaissance (all parallel, individually failable) ---
        let (analysis_res, server_info_res, form_mapping_res, api_discovery_res) = tokio::join!(
            self.scanner.scan(&url),
            self.investigator.investigate(&url),
            self.mapper.map(&url),
            self.discoverer.discover(&url)
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
        let (service_collector_res, security_audit_res) =
            tokio::join!(self.collector.collect(&url), self.auditor.audit(&url));

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
                .normalize_all(&url, a, si, api, self.dynamic_rule_engine)
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
