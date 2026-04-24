use crate::domain::entities::{ProbeEvidence, ProbeMethod, TlsSummary};
use rustls::ClientConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

/// Performs a lightweight TLS handshake against an open port.
/// Extracts certificate metadata, protocol version, cipher suite, and ALPN.
/// Uses a permissive verifier so self-signed certs don't abort the probe.
/// Does NOT send application data after the handshake.
pub async fn probe_tls(
    host: &str,
    stream: TcpStream,
    timeout: Duration,
) -> Option<(TlsSummary, ProbeEvidence)> {
    let config = build_permissive_tls_config();
    let connector = TlsConnector::from(Arc::new(config));

    let server_name = match rustls::pki_types::ServerName::try_from(host.to_string()) {
        Ok(sn) => sn,
        Err(_) => return None,
    };

    let tls_result = tokio::time::timeout(timeout, connector.connect(server_name, stream)).await;

    match tls_result {
        Ok(Ok(tls_stream)) => {
            let (_, conn) = tls_stream.get_ref();

            let protocol_version = conn.protocol_version().map(|v| format!("{:?}", v));

            let cipher_suite = conn
                .negotiated_cipher_suite()
                .map(|cs| format!("{:?}", cs.suite()));

            let alpn = conn
                .alpn_protocol()
                .map(|p| String::from_utf8_lossy(p).to_string());

            let (subject, issuer) = extract_cert_info(conn);

            let summary = TlsSummary {
                has_tls: true,
                protocol_version: protocol_version.clone(),
                cipher_suite: cipher_suite.clone(),
                subject: subject.clone(),
                issuer: issuer.clone(),
                alpn: alpn.clone(),
                sni_used: true,
            };

            let evidence = ProbeEvidence {
                method: ProbeMethod::Tls,
                raw_signal: format!(
                    "TLS handshake OK | proto={} cipher={} alpn={}",
                    protocol_version.as_deref().unwrap_or("?"),
                    cipher_suite.as_deref().unwrap_or("?"),
                    alpn.as_deref().unwrap_or("none"),
                ),
                interpretation: format!(
                    "TLS active. Subject: {}, Issuer: {}",
                    subject.as_deref().unwrap_or("unknown"),
                    issuer.as_deref().unwrap_or("unknown"),
                ),
            };

            Some((summary, evidence))
        }
        _ => None,
    }
}

fn extract_cert_info(conn: &rustls::ClientConnection) -> (Option<String>, Option<String>) {
    let certs = conn.peer_certificates();
    match certs {
        Some(chain) if !chain.is_empty() => {
            // Parse the first certificate (leaf) using rustls's DER format
            // We do a best-effort extraction from the DER-encoded certificate
            let der = &chain[0];
            let subject = extract_cn_from_der(der.as_ref(), true);
            let issuer = extract_cn_from_der(der.as_ref(), false);
            (subject, issuer)
        }
        _ => (None, None),
    }
}

/// Best-effort CN extraction from DER-encoded X.509 certificate.
/// Searches for the OID 2.5.4.3 (commonName) in subject or issuer.
fn extract_cn_from_der(der: &[u8], _is_subject: bool) -> Option<String> {
    // Simple heuristic: search for readable CN strings after the OID bytes
    // OID for CN: 55 04 03
    let cn_oid = [0x55, 0x04, 0x03];
    let der_str = der;
    let mut pos = 0;
    let mut found = Vec::new();

    while pos + 3 < der_str.len() {
        if der_str[pos] == cn_oid[0]
            && der_str[pos + 1] == cn_oid[1]
            && der_str[pos + 2] == cn_oid[2]
        {
            // Skip OID bytes, then tag+length
            let start = pos + 3;
            if start + 2 < der_str.len() {
                let _tag = der_str[start];
                let len = der_str[start + 1] as usize;
                let value_start = start + 2;
                if value_start + len <= der_str.len() {
                    if let Ok(cn) = std::str::from_utf8(&der_str[value_start..value_start + len]) {
                        found.push(cn.to_string());
                    }
                }
            }
        }
        pos += 1;
    }

    if found.len() >= 2 {
        if _is_subject {
            found.last().cloned()
        } else {
            found.first().cloned()
        }
    } else {
        found.first().cloned()
    }
}

/// Builds a permissive TLS config that accepts any certificate.
/// This is intentional for probing — we want metadata, not trust validation.
fn build_permissive_tls_config() -> ClientConfig {
    let mut config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(PermissiveVerifier))
        .with_no_client_auth();

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}

#[derive(Debug)]
struct PermissiveVerifier;

impl rustls::client::danger::ServerCertVerifier for PermissiveVerifier {
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
