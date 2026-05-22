use crate::domain::entities::*;
use reqwest::{header, Client};
use std::time::Duration;

pub struct AutonomousVerificationEngine {
    client: Client,
}

impl Default for AutonomousVerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AutonomousVerificationEngine {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        Self { client }
    }

    pub async fn verify_all(
        &self,
        target: &str,
        findings: &mut [CanonicalFinding],
        paths: &mut [AttackPath],
    ) {
        for f in findings.iter_mut() {
            if let Some(av) = self.verify_finding(target, f).await {
                f.active_verification = Some(av);
            }
        }
        for p in paths.iter_mut() {
            if let Some(av) = self.verify_path(target, p).await {
                p.active_verification = Some(av);
            }
        }
    }

    async fn verify_finding(
        &self,
        target: &str,
        f: &CanonicalFinding,
    ) -> Option<ActiveVerificationData> {
        let base_url = if target.starts_with("http") {
            target.to_string()
        } else {
            format!("https://{}", target)
        };
        let mut routes_to_test = f.affected_routes.clone();
        if routes_to_test.is_empty() {
            routes_to_test.push("/".to_string());
        }
        // Limit to 3 routes max for performance
        routes_to_test.truncate(3);

        let mut traces = Vec::new();
        let mut success_count = 0;

        let is_hsts = f.canonical_slug.contains("missing-hsts");
        let is_csp = f.canonical_slug.contains("missing-csp");

        if !is_hsts && !is_csp {
            return None; // We only verify specific tactical signatures
        }

        for route in &routes_to_test {
            let url = if route.starts_with('/') {
                format!("{}{}", base_url, route)
            } else {
                format!("{}/{}", base_url, route)
            };

            if let Ok(resp) = self.client.head(&url).send().await {
                let headers = format!("{:?}", resp.headers());

                let is_vuln = if is_hsts {
                    !resp
                        .headers()
                        .contains_key(header::STRICT_TRANSPORT_SECURITY)
                } else if is_csp {
                    !resp.headers().contains_key("content-security-policy")
                } else {
                    false
                };

                traces.push(VerificationTrace {
                    endpoint: url.clone(),
                    method: "HEAD".to_string(),
                    request_snapshot: format!("HEAD {} HTTP/1.1", url),
                    response_snapshot: headers,
                    is_successful: is_vuln,
                });

                if is_vuln {
                    success_count += 1;
                }
            }
        }

        if traces.is_empty() {
            return Some(ActiveVerificationData {
                status: VerificationStatus::VerificationFailed,
                reasoning: "Network probes failed to connect to target endpoints.".to_string(),
                reproducibility_score: 0,
                traces,
            });
        }

        let reproducibility_score = ((success_count as f32 / traces.len() as f32) * 100.0) as u8;

        let (status, reasoning) = if reproducibility_score == 100 {
            (
                VerificationStatus::VerifiedActionable,
                format!(
                    "Vulnerability is uniformly consistent across {} tested endpoints.",
                    traces.len()
                ),
            )
        } else if reproducibility_score == 0 {
            (
                VerificationStatus::VerifiedInert,
                "Vulnerability could not be reproduced. Issue is mitigated or structurally inert."
                    .to_string(),
            )
        } else {
            (VerificationStatus::PartiallyVerified, format!("Inconsistent behavior detected ({}% reproducibility). Vulnerability exists on some endpoints but mitigated on others.", reproducibility_score))
        };

        Some(ActiveVerificationData {
            status,
            reasoning,
            reproducibility_score,
            traces,
        })
    }

    async fn verify_path(&self, target: &str, p: &AttackPath) -> Option<ActiveVerificationData> {
        let base_url = if target.starts_with("http") {
            target.to_string()
        } else {
            format!("https://{}", target)
        };

        let mut auth_endpoints = p
            .shared_context
            .iter()
            .filter(|c| c.starts_with("Route: "))
            .map(|c| c.replace("Route: ", ""))
            .collect::<Vec<_>>();

        if auth_endpoints.is_empty() {
            auth_endpoints.push("/login".to_string());
        }

        let mut traces = Vec::new();
        let mut success_count = 0;

        let is_session_hijack =
            p.narrative.contains("downgrade") || p.narrative.contains("intercepts");

        if !is_session_hijack {
            return None; // Fallback
        }

        for route in auth_endpoints {
            let url = if route.starts_with('/') {
                format!("{}{}", base_url, route)
            } else {
                format!("{}/{}", base_url, route)
            };

            // For session hijack, we test if HTTP endpoint serves sensitive content without forcing HTTPS
            let http_url = url.replace("https://", "http://");

            if let Ok(resp) = self.client.get(&http_url).send().await {
                // If it doesn't redirect or HSTS isn't present
                let redirected = resp.url().as_str().starts_with("https://");
                let is_vuln = !redirected;

                traces.push(VerificationTrace {
                    endpoint: http_url.clone(),
                    method: "GET".to_string(),
                    request_snapshot: format!("GET {} HTTP/1.1", http_url),
                    response_snapshot: format!(
                        "HTTP Status: {}\nRedirected to HTTPS: {}",
                        resp.status(),
                        redirected
                    ),
                    is_successful: is_vuln,
                });

                if is_vuln {
                    success_count += 1;
                }
            }
        }

        if traces.is_empty() {
            return None;
        }

        let reproducibility_score = ((success_count as f32 / traces.len() as f32) * 100.0) as u8;

        let (status, reasoning) = if reproducibility_score == 100 {
            (VerificationStatus::VerifiedActionable, "Attack path contextually confirmed: Target endpoint accepts plaintext HTTP over sensitive routes.".to_string())
        } else if reproducibility_score == 0 {
            (VerificationStatus::VerifiedInert, "Attack path broken: Critical prerequisite (HTTP downgrade) failed because server forced HTTPS redirect.".to_string())
        } else {
            (
                VerificationStatus::PartiallyVerified,
                "Inconsistent attack surface constraints. Manual chaining required.".to_string(),
            )
        };

        Some(ActiveVerificationData {
            status,
            reasoning,
            reproducibility_score,
            traces,
        })
    }
}
