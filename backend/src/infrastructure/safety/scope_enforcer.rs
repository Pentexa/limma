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

        let target_host = url::Url::parse(target_url)
            .ok()
            .and_then(|url| url.host_str().map(ToString::to_string))
            .unwrap_or_else(|| target_url.to_string())
            .trim_start_matches("*.")
            .trim_end_matches('.')
            .to_ascii_lowercase();

        let is_in_scope = self.allowed_domains.iter().any(|domain| {
            let domain = domain
                .trim()
                .trim_start_matches("*.")
                .trim_start_matches('.')
                .trim_end_matches('.')
                .to_ascii_lowercase();

            !domain.is_empty()
                && (target_host == domain || target_host.ends_with(&format!(".{}", domain)))
        });

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

        let res = enforcer.validate("https://badexample.com/");
        assert!(res.is_err());
    }
}
