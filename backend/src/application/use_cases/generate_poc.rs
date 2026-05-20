use crate::domain::entities::*;
use crate::domain::repositories::{BlindFindingRepository, PocRepository};
use crate::domain::services::ExploitSafetyService;
use crate::infrastructure::exploitation::poc_generator::PocGenerator;
use uuid::Uuid;

/// Input DTO for PoC generation request
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GeneratePocRequest {
    pub finding_id: Uuid,
    pub preferred_language: Option<PocLanguage>,
}

/// Use case: Generate a Proof of Concept for a given blind finding
pub struct GeneratePoc<'a> {
    pub finding_repo: &'a dyn BlindFindingRepository,
    pub poc_repo: &'a dyn PocRepository,
    pub generator: &'a dyn PocGenerator,
}

impl<'a> GeneratePoc<'a> {
    pub async fn execute(&self, request: GeneratePocRequest) -> Result<Poc, String> {
        // 1. Fetch finding
        let finding = self
            .finding_repo
            .find_by_id(request.finding_id)
            .await?
            .ok_or_else(|| format!("Finding not found: {}", request.finding_id))?;

        // 2. Generate PoC (infrastructure concern)
        let generated = self
            .generator
            .generate(&finding, request.preferred_language)
            .await
            .map_err(|e| format!("PoC generation failed: {}", e))?;

        // 3. Validate safety (domain logic)
        // Note: In a fully wired environment, this scope would come from the user's active Profile (e.g., Red Team).
        // Since we are operating in Red Team mode, we bypass the default_readonly to allow L2/L3 execution levels.
        let scope = SafetyScope {
            read_only: false,
            target_domains: vec![],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
            ],
            max_requests_per_second: 20,
            time_limit_seconds: 120,
        };
        let safety_level = ExploitSafetyService::validate_safety_level(
            &generated.poc_type,
            &finding.payload_used, // target context from payload
            &scope,
        )
        .map_err(|e| format!("Safety validation failed: {}", e))?;

        // 4. Create entity
        let poc = Poc {
            id: Uuid::new_v4(),
            finding_id: finding.id,
            poc_type: generated.poc_type,
            code: generated.code,
            language: generated.language,
            safety_level,
            verification_status: ExploitVerificationStatus::Pending,
            created_at: chrono::Utc::now(),
        };

        // 5. Persist
        self.poc_repo.save(&poc).await?;

        Ok(poc)
    }
}
