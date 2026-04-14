use super::models::RuleDefinition;
use std::path::{Path, PathBuf};

/// Recursively discovers and loads all .yaml, .yml, and .json rule files
/// from the given directory. Returns (loaded_rules, errors).
pub fn load_rules_from_directory(dir: &Path) -> (Vec<RuleDefinition>, Vec<String>) {
    let mut rules = Vec::new();
    let mut errors = Vec::new();

    if !dir.exists() {
        errors.push(format!("[RuleLoader] Rules directory does not exist: {}", dir.display()));
        return (rules, errors);
    }

    let files = discover_rule_files(dir);
    tracing::info!("[RuleLoader] Discovered {} rule files in {}", files.len(), dir.display());

    for file_path in files {
        match load_single_file(&file_path) {
            Ok(rule) => {
                tracing::debug!("[RuleLoader] Loaded rule '{}' from {}", rule.id, file_path.display());
                rules.push(rule);
            }
            Err(e) => {
                let msg = format!("[RuleLoader] Failed to parse {}: {}", file_path.display(), e);
                tracing::warn!("{}", msg);
                errors.push(msg);
            }
        }
    }

    (rules, errors)
}

/// Recursively finds all .yaml, .yml, and .json files under the given directory.
fn discover_rule_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let yaml_pattern = format!("{}/**/*.yaml", dir.display());
    let yml_pattern = format!("{}/**/*.yml", dir.display());
    let json_pattern = format!("{}/**/*.json", dir.display());

    for pattern in &[yaml_pattern, yml_pattern, json_pattern] {
        if let Ok(entries) = glob::glob(pattern) {
            for entry in entries.flatten() {
                files.push(entry);
            }
        }
    }

    files.sort();
    files
}

/// Loads and parses a single rule file (YAML or JSON).
fn load_single_file(path: &Path) -> Result<RuleDefinition, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("IO error: {}", e))?;

    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "yaml" | "yml" => {
            serde_yaml::from_str::<RuleDefinition>(&content)
                .map_err(|e| format!("YAML parse error: {}", e))
        }
        "json" => {
            serde_json::from_str::<RuleDefinition>(&content)
                .map_err(|e| format!("JSON parse error: {}", e))
        }
        _ => Err(format!("Unsupported file extension: {}", ext)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_yaml_rule() {
        let dir = std::env::temp_dir().join("limma_rule_test");
        let _ = fs::create_dir_all(&dir);

        let yaml = r#"
id: "TEST-001"
name: "Test Rule"
description: "A test rule"
category: "Test"
severity: "low"
condition:
  header_missing:
    header: "x-test"
"#;
        fs::write(dir.join("test.yaml"), yaml).unwrap();

        let (rules, errors) = load_rules_from_directory(&dir);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "TEST-001");

        let _ = fs::remove_dir_all(&dir);
    }
}
