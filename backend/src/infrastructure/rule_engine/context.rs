use super::models::RuleContext;
use std::collections::HashMap;
use url::Url;

/// Builds a RuleContext from the raw scan response data.
/// All header keys are normalized to lowercase.
pub fn build_context_from_headers(
    url: &str,
    status_code: u16,
    headers: &HashMap<String, String>,
    body: Option<&str>,
) -> RuleContext {
    let normalized: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect();

    let is_https = url.starts_with("https://");

    // Parse URL for path and query
    let parsed_url =
        Url::parse(url).unwrap_or_else(|_| Url::parse("http://unknown").expect("static URL parse"));
    let path = parsed_url.path().to_lowercase();

    // Simple heuristics for semantic flags
    let is_login = path.contains("login") || path.contains("signin") || path.contains("auth");
    let is_sensitive =
        is_login || path.contains("admin") || path.contains("dashboard") || path.contains("user");
    let is_authenticated = normalized.contains_key("authorization")
        || normalized
            .get("cookie")
            .is_some_and(|c| c.contains("session"));

    RuleContext {
        url: url.to_string(),
        path,
        is_login,
        is_sensitive,
        is_authenticated,
        status_code,
        headers: normalized,
        body: body.map(|b| b.to_string()),
        is_https,
    }
}

/// Builds a RuleContext from multi-value headers (ServerInfo format).
#[allow(dead_code)]
pub fn build_context_from_multi_headers(
    url: &str,
    status_code: u16,
    raw_headers: &HashMap<String, Vec<String>>,
    body: Option<&str>,
) -> RuleContext {
    let normalized: HashMap<String, String> = raw_headers
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.join(", ")))
        .collect();

    let is_https = url.starts_with("https://");

    // Parse URL for path and query
    let parsed_url =
        Url::parse(url).unwrap_or_else(|_| Url::parse("http://unknown").expect("static URL parse"));
    let path = parsed_url.path().to_lowercase();

    // Simple heuristics for semantic flags
    let is_login = path.contains("login") || path.contains("signin") || path.contains("auth");
    let is_sensitive =
        is_login || path.contains("admin") || path.contains("dashboard") || path.contains("user");
    let is_authenticated = normalized.contains_key("authorization")
        || normalized
            .get("cookie")
            .is_some_and(|c| c.contains("session"));

    RuleContext {
        url: url.to_string(),
        path,
        is_login,
        is_sensitive,
        is_authenticated,
        status_code,
        headers: normalized,
        body: body.map(|b| b.to_string()),
        is_https,
    }
}
