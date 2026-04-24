use crate::domain::entities::{ProbeEvidence, ProbeMethod};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Detects database-like greeting/handshake signatures that servers send
/// immediately upon connection (MySQL, PostgreSQL, Redis).
/// Does NOT send any data — purely passive read of initial bytes.
pub async fn probe_greeting(
    stream: &mut TcpStream,
    port: u16,
    timeout: Duration,
) -> Option<ProbeEvidence> {
    let mut buffer = [0u8; 512];

    let read_result = tokio::time::timeout(timeout, stream.read(&mut buffer)).await;

    match read_result {
        Ok(Ok(size)) if size > 4 => {
            let raw_bytes = &buffer[..size];
            let text = String::from_utf8_lossy(raw_bytes);

            // MySQL handshake detection:
            // First 4 bytes = packet length (3) + sequence (1)
            // Then protocol version byte, then server version string
            if size > 5 && raw_bytes[4] == 0x0a {
                // Protocol version 10 = MySQL handshake
                let version_end = raw_bytes[5..]
                    .iter()
                    .position(|&b| b == 0x00)
                    .unwrap_or(size - 5);
                let version = String::from_utf8_lossy(&raw_bytes[5..5 + version_end]);
                return Some(ProbeEvidence {
                    method: ProbeMethod::Greeting,
                    raw_signal: format!("MySQL handshake v10 | version={}", version),
                    interpretation: format!("MySQL/MariaDB server detected (version: {})", version),
                });
            }

            // MySQL older protocol
            if text.contains("mysql") || text.contains("MariaDB") || text.contains("mariadb") {
                return Some(ProbeEvidence {
                    method: ProbeMethod::Greeting,
                    raw_signal: text.chars().take(80).collect(),
                    interpretation: "MySQL/MariaDB greeting detected via text match".to_string(),
                });
            }

            // PostgreSQL: when connecting, PG may send an 'R' authentication request
            // Format: byte 'R' (0x52), then int32 length, then int32 auth type
            if port == 5432 && raw_bytes[0] == b'R' && size >= 9 {
                return Some(ProbeEvidence {
                    method: ProbeMethod::Greeting,
                    raw_signal: "PostgreSQL auth request packet (R)".to_string(),
                    interpretation: "PostgreSQL server detected (auth handshake)".to_string(),
                });
            }

            // PostgreSQL: might also send 'E' for error
            if port == 5432 && (raw_bytes[0] == b'E' || raw_bytes[0] == b'N') {
                return Some(ProbeEvidence {
                    method: ProbeMethod::Greeting,
                    raw_signal: format!("PostgreSQL message type '{}'", raw_bytes[0] as char),
                    interpretation: "PostgreSQL server detected (error/notice response)"
                        .to_string(),
                });
            }

            // Redis: may send nothing on connect, but if something comes, check RESP
            if text.starts_with("+") || text.starts_with("-ERR") || text.starts_with("$") {
                return Some(ProbeEvidence {
                    method: ProbeMethod::Greeting,
                    raw_signal: text.chars().take(60).collect(),
                    interpretation: "Redis RESP protocol detected".to_string(),
                });
            }

            None
        }
        _ => None,
    }
}
