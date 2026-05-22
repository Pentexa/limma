use crate::domain::entities::{ProbeEvidence, ProbeMethod};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Reads initial bytes from an already-connected TCP stream to capture
/// service banners (SSH, FTP, SMTP, Redis, etc.).
/// Does NOT send any data — purely passive read.
pub async fn probe_banner(stream: &mut TcpStream, timeout: Duration) -> Option<ProbeEvidence> {
    let mut buffer = [0u8; 1024];

    let read_result = tokio::time::timeout(timeout, stream.read(&mut buffer)).await;

    match read_result {
        Ok(Ok(size)) if size > 0 => {
            let raw = String::from_utf8_lossy(&buffer[..size]).to_string();
            let trimmed = raw.trim().to_string();
            let interpretation = interpret_banner(&trimmed);

            Some(ProbeEvidence {
                method: ProbeMethod::Banner,
                raw_signal: trimmed.chars().take(120).collect(),
                interpretation,
            })
        }
        _ => None,
    }
}

fn interpret_banner(banner: &str) -> String {
    if banner.starts_with("SSH-") {
        return format!(
            "SSH banner detected: {}",
            banner.chars().take(40).collect::<String>()
        );
    }
    if banner.starts_with("220") && (banner.contains("FTP") || banner.contains("ftp")) {
        return "FTP server greeting (220)".to_string();
    }
    if banner.starts_with("220")
        && (banner.contains("SMTP") || banner.contains("smtp") || banner.contains("mail"))
    {
        return "SMTP server greeting (220)".to_string();
    }
    if banner.starts_with("220") {
        return format!(
            "Service greeting (220): {}",
            banner.chars().take(60).collect::<String>()
        );
    }
    if banner.starts_with("+OK") {
        return "POP3 server greeting (+OK)".to_string();
    }
    if banner.starts_with("* OK") {
        return "IMAP server greeting (* OK)".to_string();
    }
    if banner.contains("HTTP/") {
        return "HTTP response detected in banner".to_string();
    }
    if banner.starts_with("+PONG") || banner.starts_with("-ERR") || banner.starts_with("$") {
        return "Redis RESP protocol detected".to_string();
    }
    format!(
        "Unrecognized banner: {}",
        banner.chars().take(50).collect::<String>()
    )
}

/// Extracts a service name hint from a banner evidence interpretation.

pub fn service_hint_from_banner(evidence: &ProbeEvidence) -> Option<&'static str> {
    let interp = &evidence.interpretation;
    if interp.contains("SSH") {
        return Some("SSH");
    }
    if interp.contains("FTP") {
        return Some("FTP");
    }
    if interp.contains("SMTP") {
        return Some("SMTP");
    }
    if interp.contains("POP3") {
        return Some("POP3");
    }
    if interp.contains("IMAP") {
        return Some("IMAP");
    }
    if interp.contains("HTTP") {
        return Some("HTTP");
    }
    if interp.contains("Redis") {
        return Some("Redis");
    }
    None
}
