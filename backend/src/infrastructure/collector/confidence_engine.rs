use crate::domain::entities::{
    AmbiguityReason, DecisionOutcome, EvidenceItem, EvidenceKind, EvidenceStrength,
    FingerprintMatch, HttpSummary, MatchStrength, ProbeEvidence, ProbeMethod,
    ServiceCandidate, TlsSummary, ConfidenceBreakdown, DecisionTreeStep,
};

/// Well-known port-to-service mappings used as a weak base signal.
pub fn default_service_for_port(port: u16) -> Option<&'static str> {
    match port {
        21 => Some("FTP"),
        22 => Some("SSH"),
        25 | 465 | 587 => Some("SMTP"),
        53 => Some("DNS"),
        80 | 8080 => Some("HTTP"),
        110 | 995 => Some("POP3"),
        143 | 993 => Some("IMAP"),
        443 | 8443 => Some("HTTPS"),
        1433 => Some("MSSQL"),
        1521 => Some("Oracle"),
        2049 => Some("NFS"),
        3306 => Some("MySQL"),
        3389 => Some("RDP"),
        5432 => Some("PostgreSQL"),
        6379 => Some("Redis"),
        _ => None,
    }
}

/// Converts a raw ProbeEvidence into a structured EvidenceItem.
pub fn evidence_from_probe(ev: &ProbeEvidence) -> EvidenceItem {
    let (kind, strength, service) = classify_evidence(ev);
    EvidenceItem {
        kind,
        strength,
        source: ev.method.clone(),
        raw_signal: ev.raw_signal.clone(),
        interpretation: ev.interpretation.clone(),
        suggests_service: service,
        is_negative: false,
    }
}

/// Creates a port-assumption evidence item.
pub fn port_assumption_evidence(port: u16) -> Option<EvidenceItem> {
    default_service_for_port(port).map(|svc| EvidenceItem {
        kind: EvidenceKind::PortAssumption,
        strength: EvidenceStrength::Weak,
        source: ProbeMethod::PortDefault,
        raw_signal: format!("Port {} is commonly associated with {}", port, svc),
        interpretation: format!("Default port binding for {}", svc),
        suggests_service: Some(svc.to_string()),
        is_negative: false,
    })
}

/// Phase 4 decision engine: fingerprint matches are the PRIMARY interpretation layer.
/// Direct evidence is used as supplementary input when no fingerprint matches.
pub fn evaluate(
    _port: u16,
    evidence_items: &[EvidenceItem],
    tls_summary: &Option<TlsSummary>,
    http_summary: &Option<HttpSummary>,
    fingerprint_matches: &[FingerprintMatch],
) -> Vec<ServiceCandidate> {
    let mut candidates: Vec<ServiceCandidate> = Vec::new();

    // ── Layer 1: Build candidates from fingerprint matches ──
    for fp_match in fingerprint_matches {
        if fp_match.strength == MatchStrength::NoMatch {
            continue;
        }

        // Collect supporting evidence for this service
        let supporting: Vec<EvidenceItem> = evidence_items
            .iter()
            .filter(|e| {
                e.suggests_service
                    .as_ref()
                    .map(|s| s == &fp_match.service_name)
                    .unwrap_or(false)
                    && !e.is_negative
            })
            .cloned()
            .collect();

        // Collect conflicting evidence
        let conflicting: Vec<EvidenceItem> = evidence_items
            .iter()
            .filter(|e| {
                e.suggests_service
                    .as_ref()
                    .map(|s| s != &fp_match.service_name)
                    .unwrap_or(false)
                    && e.strength == EvidenceStrength::Strong
                    && !e.is_negative
            })
            .cloned()
            .collect();

        let has_conflicts = !conflicting.is_empty() || !fp_match.conflicting_rules.is_empty();

        let mut verification_trail = Vec::new();

        verification_trail.push(DecisionTreeStep {
            step: "Initial Fingerprint".to_string(),
            detail: format!("Matched fingerprint {} with {:?} coverage", fp_match.fingerprint_id, fp_match.coverage),
        });

        let mut decision = match fp_match.confidence_level {
            crate::domain::entities::FingerprintConfidence::Confirmed => {
                verification_trail.push(DecisionTreeStep {
                    step: "Protocol Validation".to_string(),
                    detail: "Strong protocol-level proof confirmed Verified status".to_string(),
                });
                DecisionOutcome::Verified
            },
            crate::domain::entities::FingerprintConfidence::High if !has_conflicts => {
                verification_trail.push(DecisionTreeStep {
                    step: "Protocol Validation".to_string(),
                    detail: "High confidence with no conflicts confirmed Verified status".to_string(),
                });
                DecisionOutcome::Verified
            },
            _ => {
                verification_trail.push(DecisionTreeStep {
                    step: "Protocol Validation".to_string(),
                    detail: "Confidence insufficient for Verification. Downgraded to Suspected".to_string(),
                });
                DecisionOutcome::Suspected
            },
        };

        let mut redirect_penalty = 0.0;
        let mut cdn_penalty = 0.0;
        let mut response_quality = 1.0;
        let mut header_reliability = fp_match.confidence;

        if fp_match.tier == crate::domain::entities::FingerprintTier::Generic
            && header_reliability > 0.6 {
                header_reliability = 0.6;
            }

        if let Some(http) = http_summary {
            if let Some(code) = http.status_code {
                if [301, 302, 307, 308].contains(&code) {
                    decision = DecisionOutcome::RoutingBehavior;
                    redirect_penalty = 0.4;
                    verification_trail.push(DecisionTreeStep {
                        step: "Routing Detection".to_string(),
                        detail: format!("Status {} mapped to RoutingBehavior with penalty", code),
                    });
                }
            }

            let cdn_indicators = ["cloudflare", "fastly", "akamai"];
            let mut is_cdn = false;
            
            if let Some(server) = &http.server_header {
                if cdn_indicators.iter().any(|c| server.to_lowercase().contains(c)) {
                    is_cdn = true;
                }
            }
            
            if !is_cdn {
                for (k, v) in &http.headers {
                    let combined = format!("{}:{}", k, v).to_lowercase();
                    if cdn_indicators.iter().any(|c| combined.contains(c)) {
                        is_cdn = true;
                        break;
                    }
                }
            }
            
            if is_cdn {
                decision = DecisionOutcome::CdnEdge;
                cdn_penalty = 0.3;
                verification_trail.push(DecisionTreeStep {
                    step: "Edge Detection".to_string(),
                    detail: "CDN/Proxy indicators found. Edge Detected - Origin Unknown".to_string(),
                });
            }
            
            if http.response_length.unwrap_or(0) == 0 {
                response_quality = 0.9;
                verification_trail.push(DecisionTreeStep {
                    step: "Response Quality".to_string(),
                    detail: "Zero response length detected, applying downgrade".to_string(),
                });
            }
        }

        let mut base_port_evidence = 0.0;
        if default_service_for_port(_port) == Some(fp_match.service_name.as_str()) {
            base_port_evidence = 0.1;
        }

        let final_score = (header_reliability - redirect_penalty - cdn_penalty) * response_quality;

        let breakdown = ConfidenceBreakdown {
            port_evidence: base_port_evidence,
            protocol_validation: if decision == DecisionOutcome::Verified { 1.0 } else { 0.5 },
            fingerprint_strength: fp_match.confidence,
            header_reliability,
            redirect_penalty,
            cdn_penalty,
            response_quality,
            final_score: final_score.max(0.0),
        };

        let ambiguity = if has_conflicts || decision == DecisionOutcome::Suspected || !fp_match.penalties.is_empty() {
            let mut conf_evs: Vec<String> = conflicting
                .iter()
                .map(|e| e.interpretation.clone())
                .chain(
                    fp_match.conflicting_rules.iter().map(|r| format!("Conflicting rule: expected {}", r.expected)),
                )
                .chain(
                    fp_match.penalties.iter().map(|p| format!("Penalty applied: -{:.2} ({})", p.amount, p.reason)),
                )
                .collect();

            if conf_evs.is_empty() {
                conf_evs.push(format!("Partial fingerprint coverage: {:?}", fp_match.coverage));
            }

            Some(AmbiguityReason {
                description: if has_conflicts {
                    format!(
                        "Fingerprint {} matched but {} conflicting signal(s) detected",
                        fp_match.fingerprint_id,
                        conflicting.len() + fp_match.conflicting_rules.len()
                    )
                } else if !fp_match.penalties.is_empty() {
                    format!("Score penalized for {} reasons", fp_match.penalties.len())
                } else {
                    format!(
                        "Fingerprint {} only weakly matched ({} rules missing)",
                        fp_match.fingerprint_id,
                        fp_match.missing_rules.len()
                    )
                },
                conflicting_evidence: conf_evs,
            })
        } else {
            None
        };

        // Determine best probe method from fingerprint evidence
        let probe_method = if fp_match.fingerprint_id.contains("tls") || fp_match.fingerprint_id.contains("https") {
            ProbeMethod::Tls
        } else if fp_match.fingerprint_id.contains("http") {
            ProbeMethod::Http
        } else if fp_match.fingerprint_id.contains("mysql") || fp_match.fingerprint_id.contains("postgresql") || fp_match.fingerprint_id.contains("redis") {
            ProbeMethod::Greeting
        } else if supporting.iter().any(|e| e.source == ProbeMethod::Banner) {
            ProbeMethod::Banner
        } else {
            ProbeMethod::PortDefault
        };

        candidates.push(ServiceCandidate {
            service_name: fp_match.service_name.clone(),
            confidence_breakdown: breakdown,
            decision,
            probe_method,
            supporting_evidence: supporting,
            conflicting_evidence: conflicting,
            reasoning: fp_match.reasoning.clone(),
            tls_summary: tls_summary.clone(),
            http_summary: http_summary.clone(),
            ambiguity,
            fingerprint_match: Some(fp_match.clone()),
            verification_trail,
        });
    }

    // ── Layer 2: Add evidence-only candidates not covered by fingerprints ──
    let fp_services: std::collections::HashSet<String> = candidates
        .iter()
        .map(|c| c.service_name.clone())
        .collect();

    for ev in evidence_items {
        if ev.is_negative {
            continue;
        }
        if let Some(ref svc) = ev.suggests_service {
            if !fp_services.contains(svc) && ev.strength != EvidenceStrength::Weak {
                // This service was suggested by evidence but no fingerprint matched
                let supporting = vec![ev.clone()];
                let score = match ev.strength {
                    EvidenceStrength::Strong => 0.35,
                    EvidenceStrength::Medium => 0.20,
                    EvidenceStrength::Weak => 0.10,
                };

                let breakdown = ConfidenceBreakdown {
            port_evidence: score,
            protocol_validation: 0.0,
            fingerprint_strength: 0.0,
            header_reliability: score,
            redirect_penalty: 0.0,
            cdn_penalty: 0.0,
            response_quality: 1.0,
            final_score: score,
        };

                candidates.push(ServiceCandidate {
                    service_name: svc.clone(),
                    confidence_breakdown: breakdown,
                    decision: DecisionOutcome::Suspected,
                    probe_method: ev.source.clone(),
                    supporting_evidence: supporting,
                    conflicting_evidence: vec![],
                    reasoning: format!(
                        "{} suggested by {} evidence but no fingerprint matched",
                        svc,
                        format!("{:?}", ev.strength).to_lowercase(),
                    ),
                    tls_summary: tls_summary.clone(),
                    http_summary: http_summary.clone(),
                    ambiguity: Some(AmbiguityReason {
                        description: "No fingerprint confirmation — evidence-only candidate".into(),
                        conflicting_evidence: vec![],
                    }),
                    fingerprint_match: None,
                    verification_trail: vec![DecisionTreeStep {
                        step: "Evidence Fallback".to_string(),
                        detail: "No fingerprint match. Marked as Suspected from evidence.".to_string(),
                    }],
                });
            }
        }
    }

    // ── Sort by confidence descending ──
    candidates.sort_by(|a, b| {
        b.confidence_breakdown.final_score
            .partial_cmp(&a.confidence_breakdown.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate by service name (keep highest confidence)
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.service_name.clone()));

    candidates
}

fn classify_evidence(ev: &ProbeEvidence) -> (EvidenceKind, EvidenceStrength, Option<String>) {
    let interp_lower = ev.interpretation.to_lowercase();

    match ev.method {
        ProbeMethod::Greeting => {
            let svc = extract_service(&interp_lower);
            (EvidenceKind::ProtocolGreeting, EvidenceStrength::Strong, svc)
        }
        ProbeMethod::Tls => {
            let svc = if interp_lower.contains("tls active") {
                Some("HTTPS".to_string())
            } else {
                None
            };
            (EvidenceKind::TlsHandshake, EvidenceStrength::Strong, svc)
        }
        ProbeMethod::Http => {
            let svc = Some("HTTP".to_string());
            (EvidenceKind::HttpResponse, EvidenceStrength::Medium, svc)
        }
        ProbeMethod::Banner => {
            let svc = extract_service(&interp_lower);
            let strength = if svc.is_some() {
                EvidenceStrength::Medium
            } else {
                EvidenceStrength::Weak
            };
            (EvidenceKind::BannerText, strength, svc)
        }
        ProbeMethod::PortDefault => {
            (EvidenceKind::PortAssumption, EvidenceStrength::Weak, None)
        }
    }
}

fn extract_service(text: &str) -> Option<String> {
    if text.contains("ssh") { return Some("SSH".into()); }
    if text.contains("ftp") { return Some("FTP".into()); }
    if text.contains("smtp") { return Some("SMTP".into()); }
    if text.contains("pop3") { return Some("POP3".into()); }
    if text.contains("imap") { return Some("IMAP".into()); }
    if text.contains("mysql") || text.contains("mariadb") { return Some("MySQL".into()); }
    if text.contains("postgresql") || text.contains("postgres") { return Some("PostgreSQL".into()); }
    if text.contains("redis") { return Some("Redis".into()); }
    if text.contains("http") { return Some("HTTP".into()); }
    None
}
