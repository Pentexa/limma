use super::models::{RuleConditionNode, RuleContext, RuleDefinition, DynamicRuleFinding, EvaluationTrace, LocalizedMessage};

/// Evaluates a single rule against the given context.
/// Returns Some(DynamicRuleFinding) if the rule matches, None otherwise.
pub fn evaluate_rule(rule: &RuleDefinition, ctx: &RuleContext) -> Option<DynamicRuleFinding> {
    let mut evidence: Vec<LocalizedMessage> = Vec::new();
    let trace = evaluate_node(&rule.condition, ctx, &mut evidence);

    if trace.is_met {
        let mut effective_severity = rule.default_severity.clone();
        let mut effective_confidence = rule.default_confidence.clone();
        let mut calibration_reasons = Vec::new();

        // ── Calibration Layer Setup ──
        if ctx.is_sensitive {
            if effective_severity == "low" { effective_severity = "medium".to_string(); }
            else if effective_severity == "medium" { effective_severity = "high".to_string(); }
            calibration_reasons.push(LocalizedMessage::new("dre.calib.sensitive_route_boost"));
        }

        if ctx.is_authenticated {
            if effective_confidence == "tentative" { effective_confidence = "firm".to_string(); }
            calibration_reasons.push(LocalizedMessage::new("dre.calib.auth_route_boost"));
        }

        Some(DynamicRuleFinding {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            category: rule.category.clone(),
            severity: effective_severity.clone(),    // Fallback/alias
            effective_severity,
            default_severity: rule.default_severity.clone(),
            effective_confidence,
            default_confidence: rule.default_confidence.clone(),
            calibration_reasons,
            rule_version: rule.version.clone(),
            rule_pack: rule.pack.clone(),
            compliance_tags: rule.compliance_tags.clone().unwrap_or_default(),
            description: LocalizedMessage::new("dre.rule.description_fallback").with_param("text", &rule.description),
            remediation: rule.remediation.as_ref().map(|r| LocalizedMessage::new("dre.rule.remediation_fallback").with_param("text", r)),
            tags: rule.tags.clone().unwrap_or_default(),
            matched_evidence: evidence,
            priority: rule.priority,
            dedup_key: rule.dedup_key.clone(),
            evaluation_trace: Some(trace),
            rule_reputation_score: None,
            feedback_summary: None,
            source: rule.source.clone(),
        })
    } else {
        None
    }
}

/// Evaluates all rules against the given context.
/// Returns a list of findings for every rule that matched.
pub fn evaluate_all(rules: &[RuleDefinition], ctx: &RuleContext) -> Vec<DynamicRuleFinding> {
    rules.iter()
        .filter_map(|rule| evaluate_rule(rule, ctx))
        .collect()
}

/// Recursively evaluates a condition node against the context.
/// Collects evidence strings into the evidence vec when leaf conditions match.
/// Returns a structured EvaluationTrace.
fn evaluate_node(node: &RuleConditionNode, ctx: &RuleContext, evidence: &mut Vec<LocalizedMessage>) -> EvaluationTrace {
    match node {
        RuleConditionNode::HeaderMissing { header } => {
            let key = header.to_lowercase();
            let missing = !ctx.headers.contains_key(&key);
            let detail = LocalizedMessage::new("dre.trace.header_missing").with_param("header", header);
            if missing { evidence.push(detail.clone()); }
            EvaluationTrace { condition_type: "header_missing".to_string(), is_met: missing, detail: Some(detail), children: None }
        }

        RuleConditionNode::HeaderPresent { header } => {
            let key = header.to_lowercase();
            let present = ctx.headers.contains_key(&key);
            let val_str = if present { ctx.headers.get(&key).unwrap().clone() } else { "".to_string() };
            let detail = LocalizedMessage::new("dre.trace.header_present").with_param("header", header).with_param("value", &truncate(&val_str, 80));
            if present { evidence.push(detail.clone()); }
            EvaluationTrace { condition_type: "header_present".to_string(), is_met: present, detail: Some(detail), children: None }
        }

        RuleConditionNode::HeaderValueContains { header, value } => {
            let key = header.to_lowercase();
            if let Some(header_val) = ctx.headers.get(&key) {
                let matches = header_val.to_lowercase().contains(&value.to_lowercase());
                let detail = LocalizedMessage::new("dre.trace.header_value_contains")
                    .with_param("header", header)
                    .with_param("expected", value)
                    .with_param("actual", &truncate(header_val, 120))
                    .with_param("matches", &matches.to_string());
                if matches { evidence.push(detail.clone()); }
                EvaluationTrace { condition_type: "header_value_contains".to_string(), is_met: matches, detail: Some(detail), children: None }
            } else {
                let detail = LocalizedMessage::new("dre.trace.header_not_found").with_param("header", header);
                EvaluationTrace { condition_type: "header_value_contains".to_string(), is_met: false, detail: Some(detail), children: None }
            }
        }

        RuleConditionNode::HeaderValueMatches { header, pattern } => {
            let key = header.to_lowercase();
            if let Some(header_val) = ctx.headers.get(&key) {
                if let Ok(re) = regex::Regex::new(pattern) {
                    let matches = re.is_match(header_val);
                    let detail = LocalizedMessage::new("dre.trace.header_value_matches")
                        .with_param("header", header)
                        .with_param("pattern", pattern)
                        .with_param("actual", &truncate(header_val, 120))
                        .with_param("matches", &matches.to_string());
                    if matches { evidence.push(detail.clone()); }
                    EvaluationTrace { condition_type: "header_value_matches".to_string(), is_met: matches, detail: Some(detail), children: None }
                } else {
                    let detail = LocalizedMessage::new("dre.trace.invalid_regex").with_param("pattern", pattern);
                    EvaluationTrace { condition_type: "header_value_matches".to_string(), is_met: false, detail: Some(detail), children: None }
                }
            } else {
                let detail = LocalizedMessage::new("dre.trace.header_not_found").with_param("header", header);
                EvaluationTrace { condition_type: "header_value_matches".to_string(), is_met: false, detail: Some(detail), children: None }
            }
        }

        RuleConditionNode::StatusCodeIn { codes } => {
            let matches = codes.contains(&ctx.status_code);
            let detail = LocalizedMessage::new("dre.trace.status_code_in")
                .with_param("code", &ctx.status_code.to_string())
                .with_param("expected", &format!("{:?}", codes));
            if matches { evidence.push(detail.clone()); }
            EvaluationTrace { condition_type: "status_code_in".to_string(), is_met: matches, detail: Some(detail), children: None }
        }

        RuleConditionNode::BodyContains { value } => {
            if let Some(ref body) = ctx.body {
                let matches = body.to_lowercase().contains(&value.to_lowercase());
                let detail = LocalizedMessage::new("dre.trace.body_contains")
                    .with_param("expected", &truncate(value, 30))
                    .with_param("matches", &matches.to_string());
                if matches { evidence.push(detail.clone()); }
                EvaluationTrace { condition_type: "body_contains".to_string(), is_met: matches, detail: Some(detail), children: None }
            } else {
                let detail = LocalizedMessage::new("dre.trace.body_empty");
                EvaluationTrace { condition_type: "body_contains".to_string(), is_met: false, detail: Some(detail), children: None }
            }
        }

        RuleConditionNode::TlsState { is_https } => {
            let matches = ctx.is_https == *is_https;
            let detail = LocalizedMessage::new("dre.trace.tls_state")
                .with_param("expected", &is_https.to_string())
                .with_param("matches", &matches.to_string());
            if matches { evidence.push(detail.clone()); }
            EvaluationTrace { condition_type: "tls_state".to_string(), is_met: matches, detail: Some(detail), children: None }
        }

        RuleConditionNode::ContextFlag { flag } => {
            let matches = match flag.as_str() {
                "is_login" => ctx.is_login,
                "is_sensitive" => ctx.is_sensitive,
                "is_authenticated" => ctx.is_authenticated,
                _ => false,
            };
            let detail = LocalizedMessage::new("dre.trace.context_flag")
                .with_param("flag", flag)
                .with_param("matches", &matches.to_string());
            if matches { evidence.push(detail.clone()); }
            EvaluationTrace { condition_type: "context_flag".to_string(), is_met: matches, detail: Some(detail), children: None }
        }

        // Logical combinators
        RuleConditionNode::All(children) => {
            let mut traces = Vec::new();
            let mut all_evidence = Vec::new();
            let mut all_match = true;
            for child in children {
                let trace = evaluate_node(child, ctx, &mut all_evidence);
                if !trace.is_met { all_match = false; }
                traces.push(trace);
            }
            if all_match { evidence.extend(all_evidence); }
            let detail = LocalizedMessage::new("dre.trace.logic_all").with_param("matches", &all_match.to_string());
            EvaluationTrace { condition_type: "all".to_string(), is_met: all_match, detail: Some(detail), children: Some(traces) }
        }

        RuleConditionNode::Any(children) => {
            let mut traces = Vec::new();
            let mut any_match = false;
            for child in children {
                let mut child_evidence = Vec::new();
                let trace = evaluate_node(child, ctx, &mut child_evidence);
                traces.push(trace.clone());
                if trace.is_met && !any_match {
                    evidence.extend(child_evidence);
                    any_match = true;
                }
            }
            let detail = LocalizedMessage::new("dre.trace.logic_any").with_param("matches", &any_match.to_string());
            EvaluationTrace { condition_type: "any".to_string(), is_met: any_match, detail: Some(detail), children: Some(traces) }
        }

        RuleConditionNode::Not(inner) => {
            let mut discard = Vec::new();
            let trace = evaluate_node(inner, ctx, &mut discard);
            let not_match = !trace.is_met;
            if not_match { evidence.push(LocalizedMessage::new("dre.trace.logic_not_met")); }
            let detail = LocalizedMessage::new("dre.trace.logic_not");
            EvaluationTrace { condition_type: "not".to_string(), is_met: not_match, detail: Some(detail), children: Some(vec![trace]) }
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}
