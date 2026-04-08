use crate::domain::entities::{
    CoverageLevel, EvidenceItem, EvidenceKind, EvidenceStrength,
    ExplanationItem, FingerprintConfidence, FingerprintDefinition,
    FingerprintMatch, FingerprintTier, HttpSummary, MatchPenalty,
    MatchStrength, RuleCategory, RuleEvaluation, RuleWeight, TlsSummary,
};

// ── Scoring constants ──
const BOOST_CRITICAL: f32 = 1.5;   // Critical rule match multiplier
const BOOST_STRONG: f32 = 1.2;     // Strong rule match multiplier
const DECAY_MISSING_REQUIRED: f32 = 0.30;  // Penalty for missing required
const DECAY_MISSING_STRONG: f32 = 0.12;    // Decay per missing strong rule
const DECAY_MISSING_MEDIUM: f32 = 0.05;    // Decay per missing medium rule
const PENALTY_CONTRADICTION: f32 = 0.15;   // Per conflicting signal
const WEAK_ONLY_CAP: f32 = 0.30;           // Max score if ONLY weak rules matched

/// Evaluates a single FingerprintDefinition against collected evidence.
/// Phase 5: non-additive scoring with boosts, decay, penalties, contextual rules,
/// and structured explanation items.
pub fn evaluate_fingerprint(
    fp: &FingerprintDefinition,
    port: u16,
    evidence: &[EvidenceItem],
    tls_summary: &Option<TlsSummary>,
    http_summary: &Option<HttpSummary>,
) -> FingerprintMatch {
    let mut matched_rules = Vec::new();
    let mut missing_rules = Vec::new();
    let mut conflicting_rules = Vec::new();
    let mut explanations: Vec<ExplanationItem> = Vec::new();
    let mut penalties: Vec<MatchPenalty> = Vec::new();

    // Track which rule categories already matched (for contextual rules)
    let mut matched_categories: Vec<RuleCategory> = Vec::new();

    let mut raw_score: f32 = 0.0;
    let mut total_possible: f32 = 0.0;
    let mut required_failed = false;
    let mut has_critical_match = false;
    let mut has_strong_match = false;
    let mut only_weak_rules = true;

    // ── Pass 1: Evaluate each rule ──
    for rule in &fp.rules {
        // Check contextual prerequisite
        let contextual_skipped = if let Some(ref prereq) = rule.contextual_requires {
            !matched_categories.contains(prereq)
        } else {
            false
        };

        if contextual_skipped {
            missing_rules.push(RuleEvaluation {
                category: rule.category.clone(),
                expected: rule.expected_value.clone(),
                actual: None,
                matched: false,
                weight: rule.weight,
                rule_weight: rule.rule_weight.clone(),
                contribution: 0.0,
                skipped_contextual: true,
            });
            explanations.push(ExplanationItem {
                category: "contextual".into(),
                description: format!("{}: skipped (prerequisite {:?} not met)", rule.description, rule.contextual_requires),
                impact: 0.0,
            });
            continue;
        }

        total_possible += rule.weight;

        let (matched, actual) = evaluate_rule(
            &rule.category,
            &rule.expected_value,
            port,
            evidence,
            tls_summary,
            http_summary,
        );

        if matched {
            // Apply weight multiplier based on rule weight
            let multiplier = match rule.rule_weight {
                RuleWeight::Critical => BOOST_CRITICAL,
                RuleWeight::Strong => BOOST_STRONG,
                RuleWeight::Medium => 1.0,
                RuleWeight::Weak => 1.0,
                RuleWeight::Contextual => 1.0,
            };
            let contribution = rule.weight * multiplier;
            raw_score += contribution;

            if rule.rule_weight == RuleWeight::Critical {
                has_critical_match = true;
                only_weak_rules = false;
                explanations.push(ExplanationItem {
                    category: "boost".into(),
                    description: format!("CRITICAL: {} (×{:.1} boost)", rule.description, multiplier),
                    impact: contribution,
                });
            } else if rule.rule_weight == RuleWeight::Strong {
                has_strong_match = true;
                only_weak_rules = false;
                explanations.push(ExplanationItem {
                    category: "boost".into(),
                    description: format!("Strong: {} (×{:.1} boost)", rule.description, multiplier),
                    impact: contribution,
                });
            } else if rule.rule_weight != RuleWeight::Weak {
                only_weak_rules = false;
                explanations.push(ExplanationItem {
                    category: "info".into(),
                    description: format!("{}: matched", rule.description),
                    impact: contribution,
                });
            }

            matched_categories.push(rule.category.clone());
            matched_rules.push(RuleEvaluation {
                category: rule.category.clone(),
                expected: rule.expected_value.clone(),
                actual,
                matched: true,
                weight: rule.weight,
                rule_weight: rule.rule_weight.clone(),
                contribution,
                skipped_contextual: false,
            });
        } else {
            // Apply decay for missing signals
            let decay = match (&rule.rule_weight, rule.required) {
                (_, true) => {
                    required_failed = true;
                    DECAY_MISSING_REQUIRED
                }
                (RuleWeight::Critical | RuleWeight::Strong, false) => DECAY_MISSING_STRONG,
                (RuleWeight::Medium, false) => DECAY_MISSING_MEDIUM,
                _ => 0.0,
            };

            if decay > 0.0 {
                raw_score -= decay;
                explanations.push(ExplanationItem {
                    category: "decay".into(),
                    description: format!("Missing: {} (expected: {})", rule.description, rule.expected_value),
                    impact: -decay,
                });
            }

            missing_rules.push(RuleEvaluation {
                category: rule.category.clone(),
                expected: rule.expected_value.clone(),
                actual,
                matched: false,
                weight: rule.weight,
                rule_weight: rule.rule_weight.clone(),
                contribution: -decay,
                skipped_contextual: false,
            });
        }
    }

    // ── Pass 2: Check for contradicting evidence ──
    for ev in evidence {
        if let Some(ref svc) = ev.suggests_service {
            if svc != &fp.service_name
                && ev.strength == EvidenceStrength::Strong
            {
                let pen = PENALTY_CONTRADICTION;
                raw_score -= pen;
                penalties.push(MatchPenalty {
                    reason: format!("Strong evidence for competing service: {}", svc),
                    amount: pen,
                });
                conflicting_rules.push(RuleEvaluation {
                    category: RuleCategory::BannerContains,
                    expected: fp.service_name.clone(),
                    actual: Some(format!("Strong evidence for {}", svc)),
                    matched: false,
                    weight: 0.15,
                    rule_weight: RuleWeight::Strong,
                    contribution: -pen,
                    skipped_contextual: false,
                });
                explanations.push(ExplanationItem {
                    category: "penalty".into(),
                    description: format!("Contradiction: strong signal for {} conflicts", svc),
                    impact: -pen,
                });
            }
        }
    }

    // ── Pass 3: Apply weak-only cap ──
    if only_weak_rules && matched_rules.len() > 0 && raw_score > WEAK_ONLY_CAP * total_possible.max(1.0) {
        let capped = WEAK_ONLY_CAP * total_possible.max(1.0);
        explanations.push(ExplanationItem {
            category: "penalty".into(),
            description: format!("Weak-only cap: score reduced from {:.2} to {:.2} (no strong/critical evidence)", raw_score, capped),
            impact: capped - raw_score,
        });
        penalties.push(MatchPenalty {
            reason: "Only weak rules matched — confidence capped".into(),
            amount: raw_score - capped,
        });
        raw_score = capped;
    }

    // ── Compute final confidence ──
    let adjusted_possible = total_possible * BOOST_CRITICAL; // theoretical max with boosts
    let confidence = if adjusted_possible > 0.0 {
        (raw_score / adjusted_possible).clamp(0.02, 0.95)
    } else {
        0.02
    };

    // ── Determine strength, coverage, confidence level ──
    let strength = if required_failed {
        MatchStrength::NoMatch
    } else {
        compute_strength(confidence, has_critical_match, has_strong_match, &matched_rules, &fp.rules)
    };

    let coverage = compute_coverage(&matched_rules, &missing_rules);
    let confidence_level = compute_confidence_level(confidence, &strength, has_critical_match, &fp.tier);

    // Add tier-based explanation
    match fp.tier {
        FingerprintTier::Specific => {
            explanations.push(ExplanationItem {
                category: "info".into(),
                description: format!("Tier: Specific ({})", fp.id),
                impact: 0.0,
            });
        }
        FingerprintTier::Generic => {
            explanations.push(ExplanationItem {
                category: "info".into(),
                description: format!("Tier: Generic fallback category ({})", fp.id),
                impact: 0.0,
            });
        }
        FingerprintTier::Fallback => {
            explanations.push(ExplanationItem {
                category: "info".into(),
                description: format!("Tier: Fallback — weak identification ({})", fp.id),
                impact: 0.0,
            });
        }
    }

    let reasoning = build_reasoning(
        &fp.service_name, &strength, &confidence_level, &coverage,
        confidence, matched_rules.len(), missing_rules.len(),
        conflicting_rules.len(), &fp.tier,
    );

    FingerprintMatch {
        fingerprint_id: fp.id.clone(),
        service_name: fp.service_name.clone(),
        tier: fp.tier.clone(),
        strength,
        confidence,
        confidence_level,
        coverage,
        matched_rules,
        missing_rules,
        conflicting_rules,
        explanation_items: explanations,
        penalties,
        reasoning,
    }
}

fn evaluate_rule(
    category: &RuleCategory,
    expected: &str,
    port: u16,
    evidence: &[EvidenceItem],
    tls_summary: &Option<TlsSummary>,
    http_summary: &Option<HttpSummary>,
) -> (bool, Option<String>) {
    match category {
        RuleCategory::BannerContains => {
            for ev in evidence {
                if ev.kind == EvidenceKind::BannerText || ev.kind == EvidenceKind::ProtocolGreeting {
                    if expected.is_empty() || ev.raw_signal.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(truncate(&ev.raw_signal, 80)));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::BannerStartsWith => {
            for ev in evidence {
                if ev.kind == EvidenceKind::BannerText || ev.kind == EvidenceKind::ProtocolGreeting {
                    if ev.raw_signal.trim().to_uppercase().starts_with(&expected.to_uppercase()) {
                        return (true, Some(truncate(&ev.raw_signal, 80)));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::TlsPresent => {
            if let Some(ref tls) = tls_summary {
                if tls.has_tls { return (true, Some("TLS active".into())); }
            }
            (false, None)
        }
        RuleCategory::TlsAlpnContains => {
            if let Some(ref tls) = tls_summary {
                if let Some(ref alpn) = tls.alpn {
                    if alpn.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(alpn.clone()));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::TlsCertSubjectContains => {
            if let Some(ref tls) = tls_summary {
                if let Some(ref subject) = tls.subject {
                    if subject.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(subject.clone()));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::HttpStatusRange => {
            if let Some(ref http) = http_summary {
                if let Some(status) = http.status_code {
                    let parts: Vec<&str> = expected.split('-').collect();
                    if parts.len() == 2 {
                        if let (Ok(lo), Ok(hi)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                            if status >= lo && status <= hi {
                                return (true, Some(format!("{}", status)));
                            }
                        }
                    }
                }
            }
            (false, None)
        }
        RuleCategory::HttpServerContains => {
            if let Some(ref http) = http_summary {
                if let Some(ref server) = http.server_header {
                    if expected.is_empty() || server.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(server.clone()));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::HttpContentTypeContains => {
            if let Some(ref http) = http_summary {
                if let Some(ref ct) = http.content_type {
                    if ct.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(ct.clone()));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::GreetingSignature => {
            for ev in evidence {
                if ev.kind == EvidenceKind::ProtocolGreeting {
                    if expected.is_empty() || ev.interpretation.to_lowercase().contains(&expected.to_lowercase()) {
                        return (true, Some(truncate(&ev.raw_signal, 80)));
                    }
                }
            }
            (false, None)
        }
        RuleCategory::PortBinding => {
            if let Ok(expected_port) = expected.parse::<u16>() {
                if port == expected_port {
                    return (true, Some(format!("{}", port)));
                }
            }
            (false, Some(format!("{}", port)))
        }
    }
}

fn compute_strength(
    confidence: f32,
    has_critical: bool,
    has_strong: bool,
    matched: &[RuleEvaluation],
    all_rules: &[crate::domain::entities::FingerprintRule],
) -> MatchStrength {
    let total = all_rules.iter().filter(|r| r.contextual_requires.is_none()).count();
    let matched_count = matched.len();

    if matched_count == total && confidence >= 0.70 {
        MatchStrength::Full
    } else if has_critical && confidence >= 0.45 {
        MatchStrength::Strong
    } else if has_strong && confidence >= 0.30 {
        MatchStrength::Strong
    } else if confidence >= 0.20 {
        MatchStrength::Partial
    } else if matched_count > 0 {
        MatchStrength::Weak
    } else {
        MatchStrength::NoMatch
    }
}

fn compute_coverage(matched: &[RuleEvaluation], missing: &[RuleEvaluation]) -> CoverageLevel {
    let total = matched.len() + missing.iter().filter(|m| !m.skipped_contextual).count();
    if total == 0 { return CoverageLevel::Minimal; }
    let ratio = matched.len() as f32 / total as f32;
    if ratio >= 0.90 { CoverageLevel::Full }
    else if ratio >= 0.65 { CoverageLevel::High }
    else if ratio >= 0.35 { CoverageLevel::Partial }
    else { CoverageLevel::Minimal }
}

fn compute_confidence_level(
    confidence: f32,
    strength: &MatchStrength,
    has_critical: bool,
    tier: &FingerprintTier,
) -> FingerprintConfidence {
    if *strength == MatchStrength::NoMatch {
        return FingerprintConfidence::Tentative;
    }
    if has_critical && confidence >= 0.55 {
        FingerprintConfidence::Confirmed
    } else if confidence >= 0.45 {
        FingerprintConfidence::High
    } else if confidence >= 0.25 {
        match tier {
            FingerprintTier::Specific => FingerprintConfidence::Medium,
            _ => FingerprintConfidence::Low,
        }
    } else {
        match tier {
            FingerprintTier::Fallback => FingerprintConfidence::Tentative,
            _ => FingerprintConfidence::Low,
        }
    }
}

fn build_reasoning(
    service: &str,
    strength: &MatchStrength,
    confidence_level: &FingerprintConfidence,
    coverage: &CoverageLevel,
    confidence: f32,
    matched: usize,
    missing: usize,
    conflicting: usize,
    tier: &FingerprintTier,
) -> String {
    let tier_label = match tier {
        FingerprintTier::Specific => "specific",
        FingerprintTier::Generic => "generic",
        FingerprintTier::Fallback => "fallback",
    };
    let strength_label = match strength {
        MatchStrength::Full => "Full",
        MatchStrength::Strong => "Strong",
        MatchStrength::Partial => "Partial",
        MatchStrength::Weak => "Weak",
        MatchStrength::NoMatch => "NoMatch",
    };
    let conf_label = match confidence_level {
        FingerprintConfidence::Confirmed => "Confirmed",
        FingerprintConfidence::High => "High",
        FingerprintConfidence::Medium => "Medium",
        FingerprintConfidence::Low => "Low",
        FingerprintConfidence::Tentative => "Tentative",
    };
    let cov_label = match coverage {
        CoverageLevel::Full => "full",
        CoverageLevel::High => "high",
        CoverageLevel::Partial => "partial",
        CoverageLevel::Minimal => "minimal",
    };
    let conflict_note = if conflicting > 0 {
        format!(", {} contradiction(s)", conflicting)
    } else {
        String::new()
    };

    format!(
        "{} [{}]: {} match, {} confidence ({:.0}%), {} coverage ({}/{} rules{}) — {}",
        service, tier_label, strength_label, conf_label,
        confidence * 100.0, cov_label, matched, matched + missing,
        conflict_note, if *strength == MatchStrength::Partial { "tentative" } else { "committed" },
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}…", &s[..max]) } else { s.to_string() }
}
