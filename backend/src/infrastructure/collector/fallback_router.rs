use crate::domain::entities::{
    ActivityEvent, ActivitySeverity, EvidenceItem, HttpSummary, PortProbeResult, PortState,
    ProbeEvidence, TlsSummary,
};
use chrono::Utc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

use super::banner_probe;
use super::confidence_engine;
use super::fingerprint_matcher;
use super::greeting_probe;
use super::http_probe;
use super::tls_probe;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const BANNER_TIMEOUT: Duration = Duration::from_millis(1500);
const TLS_TIMEOUT: Duration = Duration::from_secs(3);
const HTTP_TIMEOUT: Duration = Duration::from_secs(3);
const GREETING_TIMEOUT: Duration = Duration::from_millis(1500);
const MAX_RETRIES: u8 = 1;

/// Determines the probe strategy based on port number.
enum ProbeStrategy {
    HttpFirst,   // 80, 8080
    TlsFirst,    // 443, 8443
    BannerOnly,  // 22 (SSH sends greeting)
    GreetingFirst, // 3306, 5432, 6379
    BannerFirst, // everything else
}

fn strategy_for_port(port: u16) -> ProbeStrategy {
    match port {
        80 | 8080 => ProbeStrategy::HttpFirst,
        443 | 8443 => ProbeStrategy::TlsFirst,
        22 => ProbeStrategy::BannerOnly,
        3306 | 5432 | 6379 => ProbeStrategy::GreetingFirst,
        465 | 587 | 993 | 995 => ProbeStrategy::TlsFirst,
        _ => ProbeStrategy::BannerFirst,
    }
}

/// Probes a single port with fallback logic and retry support.
/// Returns the complete PortProbeResult and any timeline events generated.
pub async fn probe_with_fallback(
    host: &str,
    ip: &str,
    port: u16,
) -> (PortProbeResult, Vec<ActivityEvent>) {
    let mut timeline = Vec::new();
    let addr = format!("{}:{}", ip, port);
    let strategy = strategy_for_port(port);
    let probe_start = Instant::now();

    let strategy_name = match strategy {
        ProbeStrategy::HttpFirst => "HTTP-first",
        ProbeStrategy::TlsFirst => "TLS-first",
        ProbeStrategy::BannerOnly => "Banner-only",
        ProbeStrategy::GreetingFirst => "Greeting-first",
        ProbeStrategy::BannerFirst => "Banner-first",
    };

    timeline.push(ActivityEvent {
        timestamp: Utc::now(),
        severity: ActivitySeverity::Info,
        event_type: "PROBE_STRATEGY_SELECTED".to_string(),
        message: format!("Port {}: strategy={}", port, strategy_name),
        metadata: None,
    });

    // Attempt connection with retry
    let mut retry_count: u8 = 0;
    let mut fallback_used = false;
    let mut all_evidences: Vec<ProbeEvidence> = Vec::new();
    let mut tls_summary: Option<TlsSummary> = None;
    let mut http_summary: Option<HttpSummary> = None;

    // === Phase 1: Connect ===
    let connect_result = try_connect_with_retry(&addr, &mut retry_count, &mut timeline, port).await;

    let stream = match connect_result {
        Some(s) => s,
        None => {
            let duration = probe_start.elapsed().as_millis() as u64;
            let mut state = if retry_count > 0 {
                PortState::Filtered
            } else {
                PortState::Closed
            };

            let mut all_evidence = vec![];
            if let Some(port_ev) = confidence_engine::port_assumption_evidence(port) {
                all_evidence.push(port_ev);
                state = PortState::Ambiguous;
            }

            return (
                PortProbeResult {
                    port,
                    state,
                    latency_ms: None,
                    service_candidates: vec![],
                    all_evidence,
                    fingerprint_evaluations: vec![],
                    fallback_used: false,
                    retry_count,
                    probe_duration_ms: duration,
                },
                timeline,
            );
        }
    };

    let latency = probe_start.elapsed().as_millis() as u64;

    timeline.push(ActivityEvent {
        timestamp: Utc::now(),
        severity: ActivitySeverity::Info,
        event_type: "PORT_OPEN".to_string(),
        message: format!("Port {} is OPEN ({}ms)", port, latency),
        metadata: None,
    });

    // === Phase 2: Protocol-aware probing based on strategy ===
    match strategy {
        ProbeStrategy::HttpFirst => {
            // Try HTTP first, fallback to banner
            let mut stream = stream;
            if let Some((hs, ev)) = http_probe::probe_http_plain(host, &mut stream, HTTP_TIMEOUT).await {
                http_summary = Some(hs);
                all_evidences.push(ev);
                log_probe_event(&mut timeline, port, "HTTP_PROBE", "HTTP probe succeeded");
            } else {
                fallback_used = true;
                log_probe_event(&mut timeline, port, "FALLBACK_TRIGGERED", "HTTP failed, falling back to banner");
                // Reconnect for banner (original stream may be consumed)
                if let Some(mut new_stream) = try_connect(&addr).await {
                    if let Some(ev) = banner_probe::probe_banner(&mut new_stream, BANNER_TIMEOUT).await {
                        all_evidences.push(ev);
                        log_probe_event(&mut timeline, port, "BANNER_PROBE", "Banner fallback succeeded");
                    }
                }
            }
        }

        ProbeStrategy::TlsFirst => {
            // Try TLS handshake first
            if let Some((ts, ev)) = tls_probe::probe_tls(host, stream, TLS_TIMEOUT).await {
                tls_summary = Some(ts);
                all_evidences.push(ev);
                log_probe_event(&mut timeline, port, "TLS_HANDSHAKE", "TLS handshake succeeded");

                // If TLS succeeded, try HTTP over TLS
                if let Some(new_stream) = try_connect(&addr).await {
                    if let Some((ts2, _)) = tls_probe::probe_tls(host, new_stream, TLS_TIMEOUT).await {
                        // We already have TLS summary, skip
                        let _ = ts2;
                    }
                }

                // Try HTTP over a fresh TLS connection
                if let Some(new_stream) = try_connect(&addr).await {
                    let config = build_tls_config();
                    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config));
                    if let Ok(sn) = rustls::pki_types::ServerName::try_from(host.to_string()) {
                        if let Ok(Ok(mut tls_stream)) = tokio::time::timeout(TLS_TIMEOUT, connector.connect(sn, new_stream)).await {
                            if let Some((hs, ev)) = http_probe::probe_http_tls(host, &mut tls_stream, HTTP_TIMEOUT).await {
                                http_summary = Some(hs);
                                all_evidences.push(ev);
                                log_probe_event(&mut timeline, port, "HTTP_OVER_TLS", "HTTP over TLS probe succeeded");
                            }
                        }
                    }
                }
            } else {
                fallback_used = true;
                log_probe_event(&mut timeline, port, "FALLBACK_TRIGGERED", "TLS failed, falling back to HTTP plain");
                // Fallback: try plain HTTP
                if let Some(mut new_stream) = try_connect(&addr).await {
                    if let Some((hs, ev)) = http_probe::probe_http_plain(host, &mut new_stream, HTTP_TIMEOUT).await {
                        http_summary = Some(hs);
                        all_evidences.push(ev);
                    } else {
                        // Final fallback: banner
                        if let Some(mut banner_stream) = try_connect(&addr).await {
                            if let Some(ev) = banner_probe::probe_banner(&mut banner_stream, BANNER_TIMEOUT).await {
                                all_evidences.push(ev);
                                log_probe_event(&mut timeline, port, "BANNER_FALLBACK", "Banner fallback used");
                            }
                        }
                    }
                }
            }
        }

        ProbeStrategy::BannerOnly => {
            let mut stream = stream;
            if let Some(ev) = banner_probe::probe_banner(&mut stream, BANNER_TIMEOUT).await {
                all_evidences.push(ev);
                log_probe_event(&mut timeline, port, "BANNER_PROBE", "Banner probe succeeded");
            }
        }

        ProbeStrategy::GreetingFirst => {
            let mut stream = stream;
            if let Some(ev) = greeting_probe::probe_greeting(&mut stream, port, GREETING_TIMEOUT).await {
                all_evidences.push(ev);
                log_probe_event(&mut timeline, port, "GREETING_PROBE", "Greeting probe succeeded");
            } else {
                fallback_used = true;
                log_probe_event(&mut timeline, port, "FALLBACK_TRIGGERED", "Greeting failed, falling back to banner");
                if let Some(mut new_stream) = try_connect(&addr).await {
                    if let Some(ev) = banner_probe::probe_banner(&mut new_stream, BANNER_TIMEOUT).await {
                        all_evidences.push(ev);
                    }
                }
            }
        }

        ProbeStrategy::BannerFirst => {
            let mut stream = stream;
            if let Some(ev) = banner_probe::probe_banner(&mut stream, BANNER_TIMEOUT).await {
                all_evidences.push(ev);
                log_probe_event(&mut timeline, port, "BANNER_PROBE", "Banner probe succeeded");
            } else {
                fallback_used = true;
                log_probe_event(&mut timeline, port, "FALLBACK_TRIGGERED", "Banner empty, trying TLS check");
                if let Some(new_stream) = try_connect(&addr).await {
                    if let Some((ts, ev)) = tls_probe::probe_tls(host, new_stream, TLS_TIMEOUT).await {
                        tls_summary = Some(ts);
                        all_evidences.push(ev);
                        log_probe_event(&mut timeline, port, "TLS_FALLBACK", "TLS fallback succeeded");
                    }
                }
            }
        }
    }

    // === Phase 3: Evidence collection ===
    let mut evidence_items: Vec<EvidenceItem> = all_evidences
        .iter()
        .map(|ev| confidence_engine::evidence_from_probe(ev))
        .collect();

    // Add port assumption evidence
    if let Some(port_ev) = confidence_engine::port_assumption_evidence(port) {
        evidence_items.push(port_ev);
    }

    if (port == 443 || port == 8443) && tls_summary.is_none() {
        evidence_items.push(EvidenceItem {
            kind: crate::domain::entities::EvidenceKind::TlsHandshake,
            strength: crate::domain::entities::EvidenceStrength::Strong,
            source: crate::domain::entities::ProbeMethod::Tls,
            raw_signal: "TLS expected".to_string(),
            interpretation: "Missing TLS on expected HTTPS port".to_string(),
            suggests_service: Some("HTTPS".to_string()),
            is_negative: true,
        });
    }

    // === Phase 4: Fingerprint matching ===
    let fingerprint_matches = fingerprint_matcher::match_fingerprints(
        port,
        &evidence_items,
        &tls_summary,
        &http_summary,
        &mut timeline,
    );

    // === Decision engine (fingerprint-driven) ===
    let service_candidates = confidence_engine::evaluate(
        port,
        &evidence_items,
        &tls_summary,
        &http_summary,
        &fingerprint_matches,
    );

    if let Some(top) = service_candidates.first() {
        let fp_note = if top.fingerprint_match.is_some() {
            " [FP-confirmed]"
        } else {
            " [evidence-only]"
        };
        log_probe_event(
            &mut timeline,
            port,
            "SERVICE_DECIDED",
            &format!(
                "Decision: {} ({:?}, {:.0}%){} — {}",
                top.service_name,
                top.decision,
                top.confidence_breakdown.final_score * 100.0,
                fp_note,
                top.reasoning,
            ),
        );
    }

    let duration = probe_start.elapsed().as_millis() as u64;

    (
        PortProbeResult {
            port,
            state: PortState::Open,
            latency_ms: Some(latency),
            service_candidates,
            all_evidence: evidence_items,
            fingerprint_evaluations: fingerprint_matches,
            fallback_used,
            retry_count,
            probe_duration_ms: duration,
        },
        timeline,
    )
}

async fn try_connect(addr: &str) -> Option<TcpStream> {
    match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
        Ok(Ok(s)) => Some(s),
        _ => None,
    }
}

async fn try_connect_with_retry(
    addr: &str,
    retry_count: &mut u8,
    timeline: &mut Vec<ActivityEvent>,
    port: u16,
) -> Option<TcpStream> {
    // First attempt
    match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
        Ok(Ok(s)) => return Some(s),
        Ok(Err(_e)) => {
            // Connection refused — no retry
            return None;
        }
        Err(_) => {
            // Timeout — retry once
            *retry_count = 1;
            log_probe_event(
                timeline,
                port,
                "CONNECT_RETRY",
                &format!("Connection timeout, retrying (attempt {}/{})", 1, MAX_RETRIES),
            );
        }
    }

    // Retry attempt
    match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
        Ok(Ok(s)) => {
            log_probe_event(timeline, port, "CONNECT_RETRY_OK", "Retry succeeded");
            Some(s)
        }
        _ => None,
    }
}

fn log_probe_event(timeline: &mut Vec<ActivityEvent>, port: u16, event_type: &str, msg: &str) {
    timeline.push(ActivityEvent {
        timestamp: Utc::now(),
        severity: ActivitySeverity::Info,
        event_type: event_type.to_string(),
        message: format!("[Port {}] {}", port, msg),
        metadata: None,
    });
}

fn build_tls_config() -> rustls::ClientConfig {
    let mut config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(PermissiveFallbackVerifier))
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}

#[derive(Debug)]
struct PermissiveFallbackVerifier;

impl rustls::client::danger::ServerCertVerifier for PermissiveFallbackVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
