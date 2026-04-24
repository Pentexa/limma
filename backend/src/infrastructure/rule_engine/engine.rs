use super::evaluator;
use super::loader;
use super::models::{DynamicRuleFinding, LocalizedMessage, RuleContext, RuleDefinition};
use super::validator;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Internal mutable state for the engine's governance configuration
pub struct RuleEngineGovernance {
    pub disabled_rules: HashSet<String>,
    pub disabled_packs: HashSet<String>,
}

/// The DynamicRuleEngine orchestrates the full pipeline:
/// Load → Validate → Store rules at startup, then Evaluate against scan contexts.
pub struct DynamicRuleEngine {
    rules: Vec<RuleDefinition>,
    governance: RwLock<RuleEngineGovernance>,
    pub feedback_engine: Arc<super::feedback::RuleFeedbackEngine>,
    load_errors: Vec<String>,
    validation_errors: Vec<String>,
}

impl DynamicRuleEngine {
    /// Creates a new engine by loading and validating rules from the given directory.
    pub fn new(rules_dir: &str) -> Self {
        let rules_path = PathBuf::from(rules_dir);

        tracing::info!(
            "[DynamicRuleEngine] Initializing from: {}",
            rules_path.display()
        );

        // Phase 1: Load
        let (loaded_rules, load_errors) = loader::load_rules_from_directory(&rules_path);
        tracing::info!(
            "[DynamicRuleEngine] Loaded {} raw rule definitions ({} load errors)",
            loaded_rules.len(),
            load_errors.len()
        );

        // Phase 2: Validate
        let (valid_rules, validation_errors) = validator::validate_rules(loaded_rules);
        tracing::info!(
            "[DynamicRuleEngine] {} rules passed validation ({} rejected)",
            valid_rules.len(),
            validation_errors.len()
        );

        for rule in &valid_rules {
            tracing::info!(
                "[DynamicRuleEngine] ✓ Active rule: {} [{}] default_severity={}",
                rule.id,
                rule.name,
                rule.default_severity
            );
        }

        Self {
            rules: valid_rules,
            governance: RwLock::new(RuleEngineGovernance {
                disabled_rules: HashSet::new(),
                disabled_packs: HashSet::new(),
            }),
            feedback_engine: Arc::new(super::feedback::RuleFeedbackEngine::new()),
            load_errors,
            validation_errors,
        }
    }

    /// Creates an engine with no rules (for testing or fallback).
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            governance: RwLock::new(RuleEngineGovernance {
                disabled_rules: HashSet::new(),
                disabled_packs: HashSet::new(),
            }),
            feedback_engine: Arc::new(super::feedback::RuleFeedbackEngine::new()),
            load_errors: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// Returns the number of loaded rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Returns all loaded rule definitions (read-only).
    pub fn rules(&self) -> &[RuleDefinition] {
        &self.rules
    }

    /// Toggle Rule Pack
    pub fn toggle_pack(&self, pack: &str, enable: bool) {
        let mut gov = self.governance.write().unwrap();
        if enable {
            gov.disabled_packs.remove(pack);
        } else {
            gov.disabled_packs.insert(pack.to_string());
        }
    }

    /// Toggle Individual Rule
    pub fn toggle_rule(&self, rule_id: &str, enable: bool) {
        let mut gov = self.governance.write().unwrap();
        if enable {
            gov.disabled_rules.remove(rule_id);
        } else {
            gov.disabled_rules.insert(rule_id.to_string());
        }
    }

    /// Check if a rule is governed as active
    pub fn is_rule_active(&self, rule: &RuleDefinition) -> bool {
        let gov = self.governance.read().unwrap();
        if gov.disabled_packs.contains(&rule.pack) {
            return false;
        }
        if gov.disabled_rules.contains(&rule.id) {
            return false;
        }
        true
    }

    pub fn get_governance_snapshot(&self) -> (HashSet<String>, HashSet<String>) {
        let gov = self.governance.read().unwrap();
        (gov.disabled_packs.clone(), gov.disabled_rules.clone())
    }

    /// Returns any errors that occurred during loading.
    pub fn load_errors(&self) -> &[String] {
        &self.load_errors
    }

    /// Returns any errors that occurred during validation.
    pub fn validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Context-aware pre-filter: Determines if a rule should be evaluated
    /// based on the response content-type and URL path context.
    /// This prevents false positives from rules that don't apply to certain contexts.
    fn should_evaluate_rule(&self, rule: &RuleDefinition, ctx: &RuleContext) -> bool {
        // 1. Content-Type kontrolü: HTML dışı yanıtlarda belirli kuralları atla
        let content_type = ctx
            .headers
            .get("content-type")
            .map(|s| s.as_str())
            .unwrap_or("");

        if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
            // JSON/XML API yanıtlarında CSP ve X-Frame-Options zorunlu değil
            let rule_id_lower = rule.id.to_lowercase();
            let rule_cat_lower = rule.category.to_lowercase();
            if rule_id_lower.contains("csp")
                || rule_id_lower.contains("xfo")
                || rule_cat_lower.contains("csp")
                || rule_cat_lower.contains("x-frame")
            {
                tracing::debug!(
                    "[ContextFilter] Skipping rule {} — non-HTML content-type: {}",
                    rule.id,
                    content_type
                );
                return false;
            }
        }

        // 2. Safe path pattern kontrolü: Güvenli path'lerde sadece kritik kurallar çalışsın
        let url_lower = ctx.url.to_lowercase();
        let safe_patterns = [
            "/safe/",
            "/docs/",
            "/api-docs",
            "/education",
            "/tutorial",
            "/example",
        ];
        for pattern in &safe_patterns {
            if url_lower.contains(pattern) && rule.priority < 80 {
                tracing::debug!(
                    "[ContextFilter] Skipping low-priority rule {} (priority={}) on safe path: {}",
                    rule.id,
                    rule.priority,
                    ctx.url
                );
                return false;
            }
        }

        true
    }

    /// Evaluates all active rules against the given context.
    /// Returns a list of findings for every rule that matched, processing scope pre-filters and deduplication.
    pub fn evaluate(&self, ctx: &RuleContext) -> Vec<DynamicRuleFinding> {
        let mut raw_findings = Vec::new();

        for rule in &self.rules {
            if !self.is_rule_active(rule) {
                continue;
            }

            // Pre-filter: Context-aware evaluation (FP reduction)
            if !self.should_evaluate_rule(rule, ctx) {
                continue;
            }

            // Pre-filter: Check scope
            if let Some(scope) = &rule.scope {
                if !self.scope_matches(scope, ctx) {
                    continue; // Skip evaluation, scope didn't match
                }
            }

            if let Some(mut finding) = evaluator::evaluate_rule(rule, ctx) {
                // Apply feedback-driven calibration
                let rep = self.feedback_engine.get_rule_stats(&finding.rule_id);
                finding.rule_reputation_score = Some(rep.reputation_score);

                if rep.total_feedback > 2 {
                    if rep.reputation_score < 30.0 {
                        // High FP rate
                        finding.effective_confidence = "tentative".to_string();
                        finding.calibration_reasons.push(
                            LocalizedMessage::new("dre.calib.reputation_dropped")
                                .with_param("score", &format!("{:.1}", rep.reputation_score)),
                        );
                        finding.feedback_summary = Some(
                            LocalizedMessage::new("dre.calib.summary_high_fp")
                                .with_param("fp", &rep.false_positives.to_string())
                                .with_param("total", &rep.total_feedback.to_string()),
                        );
                    } else if rep.reputation_score > 80.0 {
                        // High Confirm rate
                        if finding.effective_confidence == "tentative" {
                            finding.effective_confidence = "firm".to_string();
                        } else if finding.effective_confidence == "firm" {
                            finding.effective_confidence = "certain".to_string();
                        }
                        finding.calibration_reasons.push(
                            LocalizedMessage::new("dre.calib.reputation_boosted")
                                .with_param("score", &format!("{:.1}", rep.reputation_score)),
                        );
                        finding.feedback_summary = Some(
                            LocalizedMessage::new("dre.calib.summary_high_confirm")
                                .with_param("confirmed", &rep.confirmed.to_string())
                                .with_param("total", &rep.total_feedback.to_string()),
                        );
                    }
                } else if rep.total_feedback > 0 {
                    finding.feedback_summary = Some(
                        LocalizedMessage::new("dre.calib.summary_evaluated")
                            .with_param("total", &rep.total_feedback.to_string()),
                    );
                }

                raw_findings.push(finding);
            }
        }

        // Deduplication and Prioritization
        let deduplicated = self.deduplicate_findings(raw_findings);

        if !deduplicated.is_empty() {
            tracing::info!(
                "[DynamicRuleEngine] {} dynamic rules triggered for {}",
                deduplicated.len(),
                ctx.url
            );
        }

        deduplicated
    }

    fn scope_matches(&self, scope: &super::models::RuleScope, ctx: &RuleContext) -> bool {
        if let Some(protocols) = &scope.protocols {
            let current_proto = if ctx.is_https { "https" } else { "http" };
            if !protocols.iter().any(|p| p.to_lowercase() == current_proto) {
                return false;
            }
        }

        if let Some(req_headers) = &scope.required_headers {
            for h in req_headers {
                if !ctx.headers.contains_key(&h.to_lowercase()) {
                    return false;
                }
            }
        }

        if let Some(content_types) = &scope.content_types {
            if let Some(ctx_ct) = ctx.headers.get("content-type") {
                let matches_ct = content_types
                    .iter()
                    .any(|ct| ctx_ct.to_lowercase().contains(&ct.to_lowercase()));
                if !matches_ct {
                    return false;
                }
            } else {
                return false; // required content types specified but none found
            }
        }

        true
    }

    fn deduplicate_findings(&self, findings: Vec<DynamicRuleFinding>) -> Vec<DynamicRuleFinding> {
        let mut by_dedup: std::collections::HashMap<String, Vec<DynamicRuleFinding>> =
            std::collections::HashMap::new();
        let mut active_findings = Vec::new();

        // 1. Group by dedup_key
        for f in findings {
            if let Some(key) = &f.dedup_key {
                by_dedup.entry(key.clone()).or_default().push(f);
            } else {
                active_findings.push(f);
            }
        }

        // 2. Resolve grouped by picking highest priority
        for (_, mut group) in by_dedup {
            group.sort_by(|a, b| b.priority.cmp(&a.priority)); // Descending priority
            if let Some(top) = group.into_iter().next() {
                active_findings.push(top);
            }
        }

        // 3. Process supersedes
        let mut superseded_ids = std::collections::HashSet::new();
        for f in &active_findings {
            if let Some(rule_def) = self.rules.iter().find(|r| r.id == f.rule_id) {
                if let Some(supersedes) = &rule_def.supersedes {
                    for sid in supersedes {
                        superseded_ids.insert(sid.clone());
                    }
                }
            }
        }

        active_findings
            .into_iter()
            .filter(|f| !superseded_ids.contains(&f.rule_id))
            .collect()
    }

    /// Generates log entries summarizing the engine boot state (for the normalization log).
    pub fn boot_log(&self) -> Vec<String> {
        let mut log = Vec::new();
        log.push(format!(
            "[DynamicRuleEngine] Loaded {} active rules ({} load errors, {} validation errors)",
            self.rules.len(),
            self.load_errors.len(),
            self.validation_errors.len()
        ));
        for err in &self.load_errors {
            log.push(format!("[DynamicRuleEngine] LOAD_ERR: {}", err));
        }
        for err in &self.validation_errors {
            log.push(format!("[DynamicRuleEngine] VALIDATION_ERR: {}", err));
        }
        for rule in &self.rules {
            log.push(format!(
                "[DynamicRuleEngine] Active: {} — {} [severity={}]",
                rule.id, rule.name, rule.default_severity
            ));
        }
        log
    }
}

/// Thread-safe shared reference type for the engine.
pub type SharedDynamicRuleEngine = Arc<DynamicRuleEngine>;

/// Resolves the rules directory path relative to the executable or workspace.
pub fn resolve_rules_dir() -> String {
    // Try relative to the current working directory first
    let cwd_rules = std::env::current_dir()
        .map(|d| d.join("rules"))
        .unwrap_or_else(|_| PathBuf::from("rules"));

    if cwd_rules.exists() {
        return cwd_rules.to_string_lossy().to_string();
    }

    // Fallback: relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_rules = exe_dir.join("rules");
            if exe_rules.exists() {
                return exe_rules.to_string_lossy().to_string();
            }
        }
    }

    // Default: just use "rules" and let the loader report the missing directory
    "rules".to_string()
}
