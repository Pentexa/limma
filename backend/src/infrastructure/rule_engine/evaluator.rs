use super::models::{
    DynamicRuleFinding, EvaluationTrace, LocalizedMessage, RuleConditionNode, RuleContext,
    RuleDefinition,
};

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
            if effective_severity == "low" {
                effective_severity = "medium".to_string();
            } else if effective_severity == "medium" {
                effective_severity = "high".to_string();
            }
            calibration_reasons.push(LocalizedMessage::new("dre.calib.sensitive_route_boost"));
        }

        if ctx.is_authenticated {
            if effective_confidence == "tentative" {
                effective_confidence = "firm".to_string();
            }
            calibration_reasons.push(LocalizedMessage::new("dre.calib.auth_route_boost"));
        }

        // ── Safe Context Detection (Epistemic Honesty) ──
        // If the page appears to be educational/documentation with proper security,
        // downgrade confidence to reduce false positives
        let safe_context_score = compute_safe_context_score(ctx);
        if safe_context_score >= 3 {
            // Strong safe context: downgrade confidence
            if effective_confidence == "certain" {
                effective_confidence = "firm".to_string();
                calibration_reasons.push(
                    LocalizedMessage::new("dre.calib.safe_context_downgrade")
                        .with_param("from", "certain")
                        .with_param("to", "firm")
                        .with_param("score", &safe_context_score.to_string()),
                );
            } else if effective_confidence == "firm" {
                effective_confidence = "tentative".to_string();
                calibration_reasons.push(
                    LocalizedMessage::new("dre.calib.safe_context_downgrade")
                        .with_param("from", "firm")
                        .with_param("to", "tentative")
                        .with_param("score", &safe_context_score.to_string()),
                );
            }
            // Also downgrade severity in very safe contexts (score >= 5)
            if safe_context_score >= 5 {
                if effective_severity == "high" {
                    effective_severity = "medium".to_string();
                } else if effective_severity == "medium" {
                    effective_severity = "low".to_string();
                }
                calibration_reasons.push(LocalizedMessage::new(
                    "dre.calib.safe_context_severity_downgrade",
                ));
            }
        }

        Some(DynamicRuleFinding {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            category: rule.category.clone(),
            severity: effective_severity.clone(), // Fallback/alias
            effective_severity,
            default_severity: rule.default_severity.clone(),
            effective_confidence,
            default_confidence: rule.default_confidence.clone(),
            calibration_reasons,
            rule_version: rule.version.clone(),
            rule_pack: rule.pack.clone(),
            compliance_tags: rule.compliance_tags.clone().unwrap_or_default(),
            description: LocalizedMessage::new("dre.rule.description_fallback")
                .with_param("text", &rule.description),
            remediation: rule.remediation.as_ref().map(|r| {
                LocalizedMessage::new("dre.rule.remediation_fallback").with_param("text", r)
            }),
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
    rules
        .iter()
        .filter_map(|rule| evaluate_rule(rule, ctx))
        .collect()
}

/// Recursively evaluates a condition node against the context.
/// Collects evidence strings into the evidence vec when leaf conditions match.
/// Returns a structured EvaluationTrace.
fn evaluate_node(
    node: &RuleConditionNode,
    ctx: &RuleContext,
    evidence: &mut Vec<LocalizedMessage>,
) -> EvaluationTrace {
    match node {
        RuleConditionNode::HeaderMissing { header } => {
            let key = header.to_lowercase();
            let missing = !ctx.headers.contains_key(&key);
            let detail =
                LocalizedMessage::new("dre.trace.header_missing").with_param("header", header);
            if missing {
                evidence.push(detail.clone());
            }
            EvaluationTrace {
                condition_type: "header_missing".to_string(),
                is_met: missing,
                detail: Some(detail),
                children: None,
            }
        }

        RuleConditionNode::HeaderPresent { header } => {
            let key = header.to_lowercase();
            let present = ctx.headers.contains_key(&key);
            let val_str = if present {
                ctx.headers.get(&key).unwrap().clone()
            } else {
                "".to_string()
            };
            let detail = LocalizedMessage::new("dre.trace.header_present")
                .with_param("header", header)
                .with_param("value", &truncate(&val_str, 80));
            if present {
                evidence.push(detail.clone());
            }
            EvaluationTrace {
                condition_type: "header_present".to_string(),
                is_met: present,
                detail: Some(detail),
                children: None,
            }
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
                if matches {
                    evidence.push(detail.clone());
                }
                EvaluationTrace {
                    condition_type: "header_value_contains".to_string(),
                    is_met: matches,
                    detail: Some(detail),
                    children: None,
                }
            } else {
                let detail = LocalizedMessage::new("dre.trace.header_not_found")
                    .with_param("header", header);
                EvaluationTrace {
                    condition_type: "header_value_contains".to_string(),
                    is_met: false,
                    detail: Some(detail),
                    children: None,
                }
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
                    if matches {
                        evidence.push(detail.clone());
                    }
                    EvaluationTrace {
                        condition_type: "header_value_matches".to_string(),
                        is_met: matches,
                        detail: Some(detail),
                        children: None,
                    }
                } else {
                    let detail = LocalizedMessage::new("dre.trace.invalid_regex")
                        .with_param("pattern", pattern);
                    EvaluationTrace {
                        condition_type: "header_value_matches".to_string(),
                        is_met: false,
                        detail: Some(detail),
                        children: None,
                    }
                }
            } else {
                let detail = LocalizedMessage::new("dre.trace.header_not_found")
                    .with_param("header", header);
                EvaluationTrace {
                    condition_type: "header_value_matches".to_string(),
                    is_met: false,
                    detail: Some(detail),
                    children: None,
                }
            }
        }

        RuleConditionNode::StatusCodeIn { codes } => {
            let matches = codes.contains(&ctx.status_code);
            let detail = LocalizedMessage::new("dre.trace.status_code_in")
                .with_param("code", &ctx.status_code.to_string())
                .with_param("expected", &format!("{:?}", codes));
            if matches {
                evidence.push(detail.clone());
            }
            EvaluationTrace {
                condition_type: "status_code_in".to_string(),
                is_met: matches,
                detail: Some(detail),
                children: None,
            }
        }

        RuleConditionNode::BodyContains { value } => {
            if let Some(ref body) = ctx.body {
                let body_lower = body.to_lowercase();
                let value_lower = value.to_lowercase();
                let matches = body_lower.contains(&value_lower);

                // HTML Entity awareness: detect if the match is only escaped HTML
                let is_html_tag = value.contains('<') || value.contains('>');
                let escaped_value = value.replace('<', "&lt;").replace('>', "&gt;");
                let only_escaped = !matches && body_lower.contains(&escaped_value.to_lowercase());

                let detail = if only_escaped {
                    // Body contains the escaped version only — likely educational content
                    LocalizedMessage::new("dre.trace.body_contains_escaped")
                        .with_param("expected", &truncate(value, 30))
                        .with_param("matches", "false")
                        .with_param("note", "Only HTML-escaped version found")
                } else {
                    LocalizedMessage::new("dre.trace.body_contains")
                        .with_param("expected", &truncate(value, 30))
                        .with_param("matches", &matches.to_string())
                };

                if matches {
                    evidence.push(detail.clone());
                }
                // If only escaped version exists, do NOT match — it's not a real vulnerability
                EvaluationTrace {
                    condition_type: "body_contains".to_string(),
                    is_met: matches && !(is_html_tag && only_escaped),
                    detail: Some(detail),
                    children: None,
                }
            } else {
                let detail = LocalizedMessage::new("dre.trace.body_empty");
                EvaluationTrace {
                    condition_type: "body_contains".to_string(),
                    is_met: false,
                    detail: Some(detail),
                    children: None,
                }
            }
        }

        RuleConditionNode::BodyContainsDecoded { value } => {
            if let Some(ref body) = ctx.body {
                // First check plain text
                let plain_match = body.to_lowercase().contains(&value.to_lowercase());

                if plain_match {
                    let detail = LocalizedMessage::new("dre.trace.body_contains_decoded")
                        .with_param("expected", &truncate(value, 30))
                        .with_param("matches", "true")
                        .with_param("encoding", "plaintext");
                    evidence.push(detail.clone());
                    EvaluationTrace {
                        condition_type: "body_contains_decoded".to_string(),
                        is_met: true,
                        detail: Some(detail),
                        children: None,
                    }
                } else {
                    // Check decoded layers
                    use super::encoding_detector::EncodingDetector;
                    match EncodingDetector::body_contains_decoded(body, value) {
                        Some(decoded) => {
                            let detail = LocalizedMessage::new("dre.trace.body_contains_decoded")
                                .with_param("expected", &truncate(value, 30))
                                .with_param("matches", "true")
                                .with_param("encoding", &decoded.source);
                            evidence.push(detail.clone());
                            EvaluationTrace {
                                condition_type: "body_contains_decoded".to_string(),
                                is_met: true,
                                detail: Some(detail),
                                children: None,
                            }
                        }
                        None => {
                            let detail = LocalizedMessage::new("dre.trace.body_contains_decoded")
                                .with_param("expected", &truncate(value, 30))
                                .with_param("matches", "false")
                                .with_param("encoding", "none");
                            EvaluationTrace {
                                condition_type: "body_contains_decoded".to_string(),
                                is_met: false,
                                detail: Some(detail),
                                children: None,
                            }
                        }
                    }
                }
            } else {
                let detail = LocalizedMessage::new("dre.trace.body_empty");
                EvaluationTrace {
                    condition_type: "body_contains_decoded".to_string(),
                    is_met: false,
                    detail: Some(detail),
                    children: None,
                }
            }
        }

        RuleConditionNode::TlsState { is_https } => {
            let matches = ctx.is_https == *is_https;
            let detail = LocalizedMessage::new("dre.trace.tls_state")
                .with_param("expected", &is_https.to_string())
                .with_param("matches", &matches.to_string());
            if matches {
                evidence.push(detail.clone());
            }
            EvaluationTrace {
                condition_type: "tls_state".to_string(),
                is_met: matches,
                detail: Some(detail),
                children: None,
            }
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
            if matches {
                evidence.push(detail.clone());
            }
            EvaluationTrace {
                condition_type: "context_flag".to_string(),
                is_met: matches,
                detail: Some(detail),
                children: None,
            }
        }

        // Logical combinators
        RuleConditionNode::All(children) => {
            let mut traces = Vec::new();
            let mut all_evidence = Vec::new();
            let mut all_match = true;
            for child in children {
                let trace = evaluate_node(child, ctx, &mut all_evidence);
                if !trace.is_met {
                    all_match = false;
                }
                traces.push(trace);
            }
            if all_match {
                evidence.extend(all_evidence);
            }
            let detail = LocalizedMessage::new("dre.trace.logic_all")
                .with_param("matches", &all_match.to_string());
            EvaluationTrace {
                condition_type: "all".to_string(),
                is_met: all_match,
                detail: Some(detail),
                children: Some(traces),
            }
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
            let detail = LocalizedMessage::new("dre.trace.logic_any")
                .with_param("matches", &any_match.to_string());
            EvaluationTrace {
                condition_type: "any".to_string(),
                is_met: any_match,
                detail: Some(detail),
                children: Some(traces),
            }
        }

        RuleConditionNode::Not(inner) => {
            let mut discard = Vec::new();
            let trace = evaluate_node(inner, ctx, &mut discard);
            let not_match = !trace.is_met;
            if not_match {
                evidence.push(LocalizedMessage::new("dre.trace.logic_not_met"));
            }
            let detail = LocalizedMessage::new("dre.trace.logic_not");
            EvaluationTrace {
                condition_type: "not".to_string(),
                is_met: not_match,
                detail: Some(detail),
                children: Some(vec![trace]),
            }
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

/// Computes a "safe context" score based on multiple indicators.
/// A score of 3+ indicates the page is likely educational/documentation
/// and findings should have reduced confidence.
fn compute_safe_context_score(ctx: &RuleContext) -> u32 {
    let mut score: u32 = 0;

    let body = ctx.body.as_deref().unwrap_or("");
    let body_lower = body.to_lowercase();

    // 1. Escaped HTML entities (educational content showing code examples)
    if body.contains("&lt;script&gt;")
        || body.contains("&lt;iframe&gt;")
        || body.contains("&amp;lt;")
    {
        score += 1;
    }

    // 2. Educational/documentation keywords
    let edu_keywords = [
        "educational",
        "documentation",
        "tutorial",
        "example",
        "lesson",
        "training",
        "course",
    ];
    let edu_count = edu_keywords
        .iter()
        .filter(|kw| body_lower.contains(*kw))
        .count();
    if edu_count >= 1 {
        score += 1;
    }
    if edu_count >= 3 {
        score += 1;
    } // Extra point for multiple edu keywords

    // 3. Code comment patterns (showing code examples, not actual vulnerabilities)
    if body.contains("// This is sample")
        || body.contains("// Example")
        || body.contains("/* ")
        || body.contains("```")
    {
        score += 1;
    }

    // 4. Strong security headers present (well-configured site)
    if ctx.headers.contains_key("content-security-policy") {
        score += 1;
    }
    if ctx.headers.contains_key("x-frame-options") {
        score += 1;
    }
    if ctx.headers.contains_key("strict-transport-security") {
        score += 1;
    }

    score
}
