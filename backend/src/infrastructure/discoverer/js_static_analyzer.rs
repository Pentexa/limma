use regex::Regex;
use std::collections::HashSet;

#[derive(Debug)]
pub struct AnalyzerRes {
    pub paths_with_evidence: Vec<(String, String, String, Option<usize>)>, // path, snippet, reason, line_num
    pub tech_stack: Vec<&'static str>,
}

pub struct JsStaticAnalyzer;

impl JsStaticAnalyzer {
    fn extract_snippet(content: &str, full_match: &str) -> (String, Option<usize>) {
        if let Some(block_start) = content.find(full_match) {
            let start = block_start.saturating_sub(40);
            let end = if block_start + full_match.len() + 40 < content.len() {
                block_start + full_match.len() + 40
            } else {
                content.len()
            };
            let line_num = content[..block_start].matches('\n').count() + 1;
            (
                format!("...{}...", &content[start..end].trim()),
                Some(line_num),
            )
        } else {
            ("".to_string(), None)
        }
    }

    pub fn analyze(content: &str) -> AnalyzerRes {
        let mut paths_with_evidence = Vec::new();
        let mut techs = HashSet::new();

        if let Ok(js_api_regex) = Regex::new(
            r#"(?i)(?:fetch|axios(?:\.\w+)?|XMLHttpRequest\.prototype\.open\s*\(\s*['"][^'"]+['"]\s*,|\$\.(?:ajax|get|post))\s*\(\s*['"]([^'"]+)['"]"#,
        ) {
            for cap in js_api_regex.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                if let Some(full_match) = cap.get(0) {
                    let (snippet, line_num) =
                        Self::extract_snippet(content, full_match.as_str());
                    paths_with_evidence.push((
                        path.as_str().to_string(),
                        snippet,
                        "Explicit XHR/fetch call detected".to_string(),
                        line_num,
                    ));
                    }
                }
            }
        }

        if let Ok(js_tpl_regex) = Regex::new(r#"`(/api/[^`\$]+)\$(?:\{|)[^`]+`"#) {
            for cap in js_tpl_regex.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                if let Some(full_match) = cap.get(0) {
                    let (snippet, line_num) =
                        Self::extract_snippet(content, full_match.as_str());
                    paths_with_evidence.push((
                        format!("{}[VAR]", path.as_str()),
                        snippet,
                        "Dynamic API route template literal".to_string(),
                        line_num,
                    ));
                    }
                }
            }
        }

        if let Ok(path_regex) =
            Regex::new(r#"(?i)(?:/api/v[0-9]/[a-zA-Z0-9.\-_]+|/api/[a-zA-Z0-9.\-_/]+)"#)
        {
            for cap in path_regex.captures_iter(content) {
                let Some(full_match) = cap.get(0) else { continue };
                let match_str = full_match.as_str();
                let (snippet, line_num) = Self::extract_snippet(content, match_str);
                paths_with_evidence.push((
                    match_str.to_string(),
                    snippet,
                    "Hardcoded API path pattern match".to_string(),
                    line_num,
                ));
            }
        }

        if let Ok(url_config_regex) =
            Regex::new(r#"(?i)(?:baseURL|apiUrl|API_ENDPOINT|endpoint)[\s:=]+['"]([^'"]+)['"]"#)
        {
            for cap in url_config_regex.captures_iter(content) {
                if let Some(base_path) = cap.get(1) {
                    if base_path.as_str().starts_with("http") || base_path.as_str().starts_with('/')
                    {
                    if let Some(full_match) = cap.get(0) {
                        let (snippet, line_num) =
                            Self::extract_snippet(content, full_match.as_str());
                        paths_with_evidence.push((
                            base_path.as_str().to_string(),
                            snippet,
                            "Base URL configuration syntax".to_string(),
                            line_num,
                        ));
                        }
                    }
                }
            }
        }

        if content.contains("graphql")
            || content.contains("__typename")
            || content.contains("mutation {")
        {
            techs.insert("GraphQL");
            paths_with_evidence.push((
                "/graphql".to_string(),
                "GraphQL Library/Mutation syntax detected in source JS".to_string(),
                "Implicit GraphQL inference".to_string(),
                None,
            ));
        }
        if content.contains("swagger") || content.contains("openapi") {
            techs.insert("Swagger/OpenAPI");
        }
        if content.contains("firebase") {
            techs.insert("Firebase SDK");
        }
        if content.contains("@apollo/client") || content.contains("apollo") {
            techs.insert("Apollo GraphQL");
        }
        if content.contains("trpc") {
            techs.insert("tRPC");
        }
        if content.contains("socket.io") || content.contains("WebSocket(") {
            techs.insert("WebSocket");
        }

        AnalyzerRes {
            paths_with_evidence,
            tech_stack: techs.into_iter().collect(),
        }
    }
}
