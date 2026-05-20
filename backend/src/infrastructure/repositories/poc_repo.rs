use crate::domain::entities::*;
use crate::domain::repositories::PocRepository;
use async_trait::async_trait;
use uuid::Uuid;

/// PostgreSQL implementation of PocRepository
pub struct PgPocRepository {
    pool: sqlx::PgPool,
}

impl PgPocRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PocRepository for PgPocRepository {
    async fn save(&self, poc: &Poc) -> Result<(), String> {
        let poc_type_str = format!("{:?}", poc.poc_type);
        let language_str = format!("{:?}", poc.language);
        let safety_str = format!("{:?}", poc.safety_level);
        let status_str = format!("{:?}", poc.verification_status);

        sqlx::query(
            r#"INSERT INTO pocs (id, finding_id, poc_type, code, language, safety_level, verification_status, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(poc.id)
        .bind(poc.finding_id)
        .bind(&poc_type_str)
        .bind(&poc.code)
        .bind(&language_str)
        .bind(&safety_str)
        .bind(&status_str)
        .bind(poc.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save PoC: {}", e))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Poc>, String> {
        let row = sqlx::query_as::<_, PocRow>(
            "SELECT id, finding_id, poc_type, code, language, safety_level, verification_status, created_at FROM pocs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find PoC: {}", e))?;

        Ok(row.map(|r| r.into_entity()))
    }

    async fn find_by_finding(&self, finding_id: Uuid) -> Result<Vec<Poc>, String> {
        let rows = sqlx::query_as::<_, PocRow>(
            "SELECT id, finding_id, poc_type, code, language, safety_level, verification_status, created_at FROM pocs WHERE finding_id = $1 ORDER BY created_at DESC",
        )
        .bind(finding_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find PoCs by finding: {}", e))?;

        Ok(rows.into_iter().map(|r| r.into_entity()).collect())
    }

    async fn update_verification(
        &self,
        id: Uuid,
        status: ExploitVerificationStatus,
    ) -> Result<(), String> {
        let status_str = format!("{:?}", status);
        sqlx::query("UPDATE pocs SET verification_status = $1 WHERE id = $2")
            .bind(&status_str)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update PoC verification: {}", e))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct PocRow {
    id: Uuid,
    finding_id: Uuid,
    poc_type: String,
    code: String,
    language: String,
    safety_level: String,
    verification_status: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PocRow {
    fn into_entity(self) -> Poc {
        Poc {
            id: self.id,
            finding_id: self.finding_id,
            poc_type: parse_poc_type(&self.poc_type),
            code: self.code,
            language: parse_poc_language(&self.language),
            safety_level: parse_safety_level(&self.safety_level),
            verification_status: parse_exploit_verification_status(&self.verification_status),
            created_at: self.created_at,
        }
    }
}

fn parse_poc_type(s: &str) -> PocType {
    match s {
        "SqlInjection" => PocType::SqlInjection,
        "CommandInjection" => PocType::CommandInjection,
        "PathTraversal" => PocType::PathTraversal,
        "ServerSideRequestForgery" => PocType::ServerSideRequestForgery,
        "XmlExternalEntity" => PocType::XmlExternalEntity,
        "InsecureDeserialization" => PocType::InsecureDeserialization,
        "CrossSiteScripting" => PocType::CrossSiteScripting,
        _ => PocType::SqlInjection,
    }
}

fn parse_poc_language(s: &str) -> PocLanguage {
    match s {
        "Python" => PocLanguage::Python,
        "Ruby" => PocLanguage::Ruby,
        "JavaScript" => PocLanguage::JavaScript,
        "Bash" => PocLanguage::Bash,
        "Rust" => PocLanguage::Rust,
        _ => PocLanguage::Python,
    }
}

fn parse_safety_level(s: &str) -> SafetyLevel {
    match s {
        "L1SafeReadOnly" => SafetyLevel::L1SafeReadOnly,
        "L2VerifiedSandbox" => SafetyLevel::L2VerifiedSandbox,
        "L3ActiveWithConsent" => SafetyLevel::L3ActiveWithConsent,
        _ => SafetyLevel::L1SafeReadOnly,
    }
}

fn parse_exploit_verification_status(s: &str) -> ExploitVerificationStatus {
    match s {
        "Pending" => ExploitVerificationStatus::Pending,
        "VerifiedInSandbox" => ExploitVerificationStatus::VerifiedInSandbox,
        "FailedVerification" => ExploitVerificationStatus::FailedVerification,
        "VerifiedInProduction" => ExploitVerificationStatus::VerifiedInProduction,
        _ => ExploitVerificationStatus::Pending,
    }
}
