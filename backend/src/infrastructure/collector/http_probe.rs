use crate::domain::entities::{HttpSummary, ProbeEvidence, ProbeMethod};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Sends a minimal HTTP GET request over a raw TCP stream and parses
/// the response status line and key headers.
/// Does NOT follow redirects, but calculates response length.
pub async fn probe_http_plain(
    host: &str,
    stream: &mut TcpStream,
    timeout: Duration,
) -> Option<(HttpSummary, ProbeEvidence)> {
    let request = format!(
        "GET / HTTP/1.0\r\nHost: {}\r\nUser-Agent: LimmaCollector/4.0\r\nConnection: close\r\n\r\n",
        host
    );

    if tokio::time::timeout(timeout, stream.write_all(request.as_bytes())).await.is_err() {
        return None;
    }

    let mut buffer = [0u8; 2048];
    let read_result = tokio::time::timeout(timeout, stream.read(&mut buffer)).await;

    match read_result {
        Ok(Ok(size)) if size > 0 => {
            let response = String::from_utf8_lossy(&buffer[..size]).to_string();
            parse_http_response(&response)
        }
        _ => None,
    }
}

/// Sends a minimal HTTP GET request over a TLS stream.
pub async fn probe_http_tls(
    host: &str,
    tls_stream: &mut tokio_rustls::client::TlsStream<TcpStream>,
    timeout: Duration,
) -> Option<(HttpSummary, ProbeEvidence)> {
    let request = format!(
        "GET / HTTP/1.0\r\nHost: {}\r\nUser-Agent: LimmaCollector/4.0\r\nConnection: close\r\n\r\n",
        host
    );

    if tokio::time::timeout(timeout, tls_stream.write_all(request.as_bytes())).await.is_err() {
        return None;
    }

    let mut buffer = [0u8; 2048];
    let read_result = tokio::time::timeout(timeout, tls_stream.read(&mut buffer)).await;

    match read_result {
        Ok(Ok(size)) if size > 0 => {
            let response = String::from_utf8_lossy(&buffer[..size]).to_string();
            parse_http_response(&response)
        }
        _ => None,
    }
}

fn parse_http_response(response: &str) -> Option<(HttpSummary, ProbeEvidence)> {
    let mut parts = response.splitn(2, "\r\n\r\n");
    let top = parts.next()?;
    let body = parts.next().unwrap_or("");

    let mut lines = top.lines();
    let status_line = lines.next()?;
    if !status_line.contains("HTTP/") {
        return None;
    }

    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok());

    let mut server_header = None;
    let mut content_type = None;
    let mut redirect_target = None;
    let mut headers = std::collections::HashMap::new();

    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            if key == "server" {
                server_header = Some(val.clone());
            } else if key == "content-type" {
                content_type = Some(val.clone());
            } else if key == "location" {
                redirect_target = Some(val.clone());
            }
            headers.insert(key, val);
        }
    }

    let response_length = headers.get("content-length")
        .and_then(|l| l.parse::<u64>().ok())
        .or({
            if !body.is_empty() {
                Some(body.len() as u64)
            } else {
                None
            }
        });

    let summary = HttpSummary {
        status_code,
        server_header: server_header.clone(),
        content_type: content_type.clone(),
        redirect_target: redirect_target.clone(),
        headers,
        response_length,
    };

    let raw = format!(
        "HTTP {} | Server: {} | Type: {}",
        status_code.map(|c| c.to_string()).unwrap_or("?".into()),
        server_header.as_deref().unwrap_or("?"),
        content_type.as_deref().unwrap_or("?"),
    );

    let interpretation = if let Some(code) = status_code {
        if (200..400).contains(&code) {
            format!("HTTP service confirmed (status {})", code)
        } else if (300..400).contains(&code) {
            format!(
                "HTTP redirect (status {}) → {}",
                code,
                redirect_target.as_deref().unwrap_or("unknown")
            )
        } else {
            format!("HTTP error response (status {})", code)
        }
    } else {
        "HTTP response detected but status unclear".to_string()
    };

    let evidence = ProbeEvidence {
        method: ProbeMethod::Http,
        raw_signal: raw,
        interpretation,
    };

    Some((summary, evidence))
}
