use crate::domain::engine_config::EngineConfig;
use crate::domain::entities::{SettingsProfile, SubdomainDiscoveryResult};
use crate::domain::repositories::SubdomainDiscoverer;

pub struct DiscoverSubdomains<'a, D: SubdomainDiscoverer> {
    pub discoverer: &'a D,
}

impl<'a, D: SubdomainDiscoverer> DiscoverSubdomains<'a, D> {
    pub async fn execute(
        &self,
        domain: String,
        profile: &SettingsProfile,
    ) -> Result<SubdomainDiscoveryResult, String> {
        let config = EngineConfig::from_profile(profile);
        self.discoverer.discover_subdomains(&domain, &config).await
    }
}
