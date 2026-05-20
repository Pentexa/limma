use crate::domain::active_vuln::ActiveVulnType;

/// Determines which vulnerability types should be prioritized based on the parameter name.
/// Returns a list of highly relevant vulnerability types. If the list is empty, all requested
/// types should be tested, or a baseline scan can be performed.
pub fn prioritize_vuln_types(param_name: &str) -> Vec<ActiveVulnType> {
    let mut prioritized = Vec::new();
    let name = param_name.to_lowercase();

    // LFI / Path Traversal Indicators
    if name.contains("file")
        || name.contains("path")
        || name.contains("dir")
        || name.contains("doc")
        || name.contains("folder")
        || name.contains("template")
        || name.contains("page")
        || name.contains("include")
    {
        prioritized.push(ActiveVulnType::LocalFileInclusion);
        prioritized.push(ActiveVulnType::PathTraversal);
        prioritized.push(ActiveVulnType::RemoteFileInclusion);
        prioritized.push(ActiveVulnType::XmlExternalEntity); // XXE often related to file uploads/doc imports
    }

    // SSRF / Open Redirect Indicators
    if name.contains("url")
        || name.contains("uri")
        || name.contains("redirect")
        || name.contains("next")
        || name.contains("dest")
        || name.contains("target")
        || name.contains("callback")
        || name.contains("host")
        || name.contains("return")
    {
        prioritized.push(ActiveVulnType::ServerSideRequestForgery);
        prioritized.push(ActiveVulnType::OpenRedirect);
    }

    // SQLi / NoSQLi Indicators
    if name.contains("id")
        || name.contains("user")
        || name.contains("query")
        || name.contains("search")
        || name.contains("sort")
        || name.contains("filter")
        || name.contains("order")
        || name.contains("name")
        || name.contains("cat")
    {
        prioritized.push(ActiveVulnType::SqlInjectionError);
        prioritized.push(ActiveVulnType::SqlInjectionUnion);
        prioritized.push(ActiveVulnType::SqlInjectionBlindTime);
        prioritized.push(ActiveVulnType::SqlInjectionBlindBoolean);
        prioritized.push(ActiveVulnType::NoSqlInjection);
    }

    // Command Injection Indicators
    if name.contains("cmd")
        || name.contains("exec")
        || name.contains("command")
        || name.contains("run")
        || name.contains("daemon")
        || name.contains("ping")
        || name.contains("ip")
        || name.contains("host")
    {
        prioritized.push(ActiveVulnType::CommandInjection);
        prioritized.push(ActiveVulnType::CommandInjectionBlind);
    }

    // Always include XSS/SSTI as a baseline if it's a generic parameter
    // (Actual reflection check is done dynamically at runtime)
    prioritized.push(ActiveVulnType::ReflectedXss);
    prioritized.push(ActiveVulnType::ServerSideTemplateInjection);

    // Remove duplicates
    prioritized.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    prioritized.dedup();

    prioritized
}
