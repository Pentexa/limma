use super::models::{RuleConditionNode, RuleDefinition};
use std::collections::HashSet;

/// Validates a batch of loaded rules for correctness.
/// Returns (valid_rules, validation_errors).
pub fn validate_rules(rules: Vec<RuleDefinition>) -> (Vec<RuleDefinition>, Vec<String>) {
    let mut valid = Vec::new();
    let mut errors = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for rule in rules {
        let mut rule_errors = Vec::new();

        // Required fields
        if rule.id.trim().is_empty() {
            rule_errors.push("Rule missing required 'id' field".to_string());
        }
        if rule.name.trim().is_empty() {
            rule_errors.push(format!("Rule '{}' missing required 'name' field", rule.id));
        }
        if rule.default_severity.trim().is_empty() {
            rule_errors.push(format!(
                "Rule '{}' missing required 'default_severity' (or 'severity') field",
                rule.id
            ));
        }

        // Validate severity value
        let valid_severities = ["critical", "high", "medium", "low", "informational"];
        if !valid_severities.contains(&rule.default_severity.to_lowercase().as_str()) {
            rule_errors.push(format!(
                "Rule '{}' has invalid severity '{}'. Must be one of: {:?}",
                rule.id, rule.default_severity, valid_severities
            ));
        }

        // Duplicate ID check
        if seen_ids.contains(&rule.id) {
            rule_errors.push(format!("Duplicate rule ID detected: '{}'", rule.id));
        }

        // Validate regex patterns in the condition tree
        if let Err(regex_errors) = validate_condition_regexes(&rule.condition) {
            for re in regex_errors {
                rule_errors.push(format!("Rule '{}': {}", rule.id, re));
            }
        }

        // Skip disabled rules silently
        if !rule.enabled {
            tracing::info!("[RuleValidator] Skipping disabled rule: {}", rule.id);
            continue;
        }

        if rule_errors.is_empty() {
            seen_ids.insert(rule.id.clone());
            valid.push(rule);
        } else {
            for err in rule_errors {
                tracing::warn!("[RuleValidator] {}", err);
                errors.push(err);
            }
        }
    }

    tracing::info!(
        "[RuleValidator] Validation complete: {} valid, {} rejected",
        valid.len(),
        errors.len()
    );

    (valid, errors)
}

/// Recursively checks all regex patterns in the condition tree for validity.
fn validate_condition_regexes(node: &RuleConditionNode) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    check_regexes_recursive(node, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_regexes_recursive(node: &RuleConditionNode, errors: &mut Vec<String>) {
    match node {
        RuleConditionNode::HeaderValueMatches { header, pattern } => {
            if let Err(e) = regex::Regex::new(pattern) {
                errors.push(format!(
                    "Invalid regex pattern '{}' for header '{}': {}",
                    pattern, header, e
                ));
            }
        }
        RuleConditionNode::All(children) | RuleConditionNode::Any(children) => {
            for child in children {
                check_regexes_recursive(child, errors);
            }
        }
        RuleConditionNode::Not(inner) => {
            check_regexes_recursive(inner, errors);
        }
        // Leaf conditions without regex - no validation needed
        _ => {}
    }
}
