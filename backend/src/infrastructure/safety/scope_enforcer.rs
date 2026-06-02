/// Scope enforcer — validates targets are within the allowed domain list.
///
/// If no domains are configured, all targets are allowed (open scope).
pub struct ScopeEnforcer {
    allowed_domains: Vec<String>,
}

impl ScopeEnforcer {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self { allowed_domains }
    }

    /// Validate that a target URL is within the allowed scope
    pub fn validate(&self, target_url: &str) -> Result<(), String> {
        // Open scope: if no domains configured, everything is allowed
        if self.allowed_domains.is_empty() {
            return Ok(());
        }

        // Check if target URL matches any allowed domain
        let url_lower = target_url.to_lowercase();
        let is_in_scope = self
            .allowed_domains
            .iter()
            .any(|domain| url_lower.contains(&domain.to_lowercase()));

        if is_in_scope {
            Ok(())
        } else {
            Err(format!(
                "Target '{}' is out of scope. Allowed domains: {:?}",
                target_url, self.allowed_domains
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_scope_allows_all() {
        let enforcer = ScopeEnforcer::new(vec![]);
        assert!(enforcer.validate("https://google.com").is_ok());
        assert!(enforcer.validate("http://evil.com/malware").is_ok());
    }

    #[test]
    fn test_strict_scope_allows_valid_domains() {
        let enforcer = ScopeEnforcer::new(vec!["example.com".to_string()]);
        assert!(enforcer.validate("https://api.example.com/v1").is_ok());
        assert!(enforcer.validate("http://example.com/").is_ok());
    }

    #[test]
    fn test_strict_scope_rejects_invalid_domains() {
        let enforcer = ScopeEnforcer::new(vec!["example.com".to_string()]);
        let res = enforcer.validate("https://evil.com/");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("out of scope"));
    }
}
