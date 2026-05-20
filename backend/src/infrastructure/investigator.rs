use crate::domain::entities::{
    CertaintyLevel, CertaintyNote, DeliveryInsight, InfrastructureSignal, InvestigatorFingerprint,
    SecurityPostureInsight, ServerInfo,
};
use crate::domain::repositories::ServerInvestigator;
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Instant;

pub struct HttpInvestigator {}

impl HttpInvestigator {
    pub fn new() -> Self {
        Self {}
    }

    fn build_client(&self, profile: &crate::domain::engine_config::EngineConfig) -> Client {
        let mut builder = Client::builder()
            .user_agent(&profile.user_agent)
            .timeout(std::time::Duration::from_millis(profile.timeout_ms))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .tcp_keepalive(std::time::Duration::from_secs(15));
            
        if profile.use_proxy {
            if let Some(proxy_url) = &profile.proxy_url {
                if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }

        builder.build().expect("Failed to build profile-specific HTTP client")
    }
}

#[async_trait]
impl ServerInvestigator for HttpInvestigator {
    async fn investigate(
        &self,
        url_str: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<ServerInfo, String> {
        let client = self.build_client(profile);
        self.investigate_multi_inner(url_str, &client, profile, &mut None).await
    }

    async fn investigate_stream(
        &self,
        url_str: &str,
        profile: &crate::domain::engine_config::EngineConfig,
        tx: tokio::sync::mpsc::UnboundedSender<crate::domain::entities::InvestigationEvent>,
    ) -> Result<ServerInfo, String> {
        let mut tx_opt = Some(tx.clone());
        let client = self.build_client(profile);
        let res = self.investigate_multi_inner(url_str, &client, profile, &mut tx_opt).await;
        if let Ok(ref info) = res {
            let event = crate::domain::entities::InvestigationEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                event_type: "INVESTIGATION_COMPLETED".to_string(),
                message: "Investigation finalized successfully".to_string(),
                payload: Some(serde_json::to_value(info).unwrap_or(serde_json::Value::Null)),
            };
            let _ = tx.send(event);
        }
        res
    }
}

impl HttpInvestigator {
    async fn investigate_multi_inner(
        &self,
        url_str: &str,
        client: &Client,
        profile: &crate::domain::engine_config::EngineConfig,
        tx: &mut Option<
            tokio::sync::mpsc::UnboundedSender<crate::domain::entities::InvestigationEvent>,
        >,
    ) -> Result<ServerInfo, String> {
        if let Some(t) = tx {
            let _ = t.send(crate::domain::entities::InvestigationEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                event_type: "INVESTIGATION_STARTED".to_string(),
                message: format!("Initiated tracking on primary target: {}", url_str),
                payload: None,
            });
        }

        // clone sender for analyze_route to avoid borrowing issues inside futures or loops
        let mut primary_info = self.analyze_route(url_str, client, profile, tx.clone()).await?;

        let base_url = if let Ok(parsed) = reqwest::Url::parse(&primary_info.resolved_url) {
            let port_str = if let Some(p) = parsed.port_or_known_default() {
                format!(":{}", p)
            } else {
                "".to_string()
            };
            format!(
                "{}://{}{}",
                parsed.scheme(),
                parsed.host_str().unwrap_or(""),
                port_str
            )
        } else {
            return Ok(primary_info);
        };

        let common_paths = vec!["/login", "/admin", "/contact"];
        let mut targets = Vec::new();
        for path in common_paths {
            targets.push(format!("{}{}", base_url, path));
        }

        let mut routes_checked = vec![primary_info.resolved_url.clone()];
        let mut responses = Vec::new();

        for target in targets {
            if let Ok(info) = self.analyze_route(&target, client, profile, tx.clone()).await {
                routes_checked.push(info.resolved_url.clone());
                responses.push(info);
            }
        }

        let mut consistency_insights: Vec<crate::domain::entities::ConsistencyInsight> = Vec::new();

        // Security check: CSP Consistency
        let primary_has_csp = primary_info
            .security_insights
            .iter()
            .any(|s| s.name.contains("Content-Security-Policy") && s.status == "Secure");
        let mut missing_csp_routes = Vec::new();
        for resp in &responses {
            let has_csp = resp
                .security_insights
                .iter()
                .any(|s| s.name.contains("Content-Security-Policy") && s.status == "Secure");
            if primary_has_csp && !has_csp {
                missing_csp_routes.push(resp.resolved_url.clone());
            }
        }
        if !missing_csp_routes.is_empty() {
            consistency_insights.push(crate::domain::entities::ConsistencyInsight {
                name: "Inconsistent CSP Protection".to_string(),
                severity: "High".to_string(),
                category: "Security Header Consistency".to_string(),
                evidences: missing_csp_routes,
                explanation: "The primary route enforces a strict Content-Security-Policy, but several sub-routes completely lack this protection, exposing them to XSS attacks.".to_string(),
            });
        }

        // Check cache / edge boundaries
        let primary_edge = primary_info.delivery_insights.iter().any(|s| {
            s.name.contains("Edge") || s.name.contains("Cloudflare") || s.name.contains("Vercel")
        });
        let mut missing_edge_routes = Vec::new();
        for resp in &responses {
            let has_edge = resp.delivery_insights.iter().any(|s| {
                s.name.contains("Edge")
                    || s.name.contains("Cloudflare")
                    || s.name.contains("Vercel")
            });
            if primary_edge && !has_edge {
                missing_edge_routes.push(resp.resolved_url.clone());
            }
        }
        if !missing_edge_routes.is_empty() {
            consistency_insights.push(crate::domain::entities::ConsistencyInsight {
                name: "Origin Routing Bypass".to_string(),
                severity: "Medium".to_string(),
                category: "Cache Consistency".to_string(),
                evidences: missing_edge_routes,
                explanation: "While the primary domain is strongly proxied behind an Edge layer, secondary routes bypass this protective CDN and hit the origin directly.".to_string(),
            });
        }

        primary_info.routes_checked = routes_checked;
        primary_info.consistency_insights = consistency_insights.clone();

        if let Some(t) = tx {
            let _ = t.send(crate::domain::entities::InvestigationEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                event_type: "ROUTE_COMPARED".to_string(),
                message: format!(
                    "Cross-request comparison executed across {} routes",
                    primary_info.routes_checked.len()
                ),
                payload: Some(
                    serde_json::to_value(&consistency_insights).unwrap_or(serde_json::Value::Null),
                ),
            });
        }

        Ok(primary_info)
    }

    async fn analyze_route(
        &self,
        url_str: &str,
        client: &Client,
        _profile: &crate::domain::engine_config::EngineConfig,
        tx: Option<tokio::sync::mpsc::UnboundedSender<crate::domain::entities::InvestigationEvent>>,
    ) -> Result<ServerInfo, String> {
        let mut activity_log = Vec::new();
        activity_log.push(format!("Resolving target: {}", url_str));

        let start_time = Instant::now();
        activity_log.push("Fetching response".to_string());

        let resp = client.get(url_str).send().await.map_err(|e| {
            activity_log.push(format!("Failed to fetch response: {}", e));
            e.to_string()
        })?;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let final_url = resp.url().to_string();
        let status_code = resp.status().as_u16();

        activity_log.push("Normalizing headers".to_string());

        let mut raw_headers: HashMap<String, Vec<String>> = HashMap::new();
        for (name, value) in resp.headers() {
            let key = name.as_str().to_lowercase();
            if let Ok(val_str) = value.to_str() {
                raw_headers
                    .entry(key)
                    .or_default()
                    .push(val_str.to_string());
            }
        }

        if let Some(t) = &tx {
            let _ = t.send(crate::domain::entities::InvestigationEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                event_type: "HEADERS_NORMALIZED".to_string(),
                message: format!(
                    "Captured & Normalized {} headers from {}",
                    raw_headers.len(),
                    final_url
                ),
                payload: Some(
                    serde_json::to_value(&raw_headers).unwrap_or(serde_json::Value::Null),
                ),
            });
        }

        activity_log.push("Categorizing headers".to_string());

        let mut categorized_headers: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        let categories = vec![
            (
                "Server/Platform",
                vec!["server", "x-powered-by", "host", "via", "x-aspnet-version"],
            ),
            (
                "Cache",
                vec![
                    "cache-control",
                    "pragma",
                    "expires",
                    "etag",
                    "last-modified",
                    "age",
                    "x-cache",
                    "cf-cache-status",
                    "x-vercel-cache",
                ],
            ),
            (
                "Security",
                vec![
                    "strict-transport-security",
                    "content-security-policy",
                    "x-frame-options",
                    "x-content-type-options",
                    "x-xss-protection",
                    "referrer-policy",
                ],
            ),
            (
                "CORS",
                vec![
                    "access-control-allow-origin",
                    "access-control-allow-methods",
                    "access-control-allow-headers",
                    "access-control-expose-headers",
                    "access-control-allow-credentials",
                ],
            ),
            (
                "Content Metadata",
                vec![
                    "content-type",
                    "content-length",
                    "content-encoding",
                    "content-language",
                    "accept-ranges",
                ],
            ),
            (
                "Proxy/CDN",
                vec![
                    "x-forwarded-for",
                    "x-real-ip",
                    "cf-ray",
                    "x-amz-cf-id",
                    "x-edge",
                    "x-cdn",
                    "x-vercel-id",
                ],
            ),
        ];

        for (key, values) in &raw_headers {
            let mut categorized = false;
            for (cat_name, cat_keys) in &categories {
                if cat_keys.contains(&key.as_str()) {
                    categorized_headers
                        .entry(cat_name.to_string())
                        .or_default()
                        .insert(key.clone(), values.clone());
                    categorized = true;
                    break;
                }
            }
            if !categorized {
                categorized_headers
                    .entry("Other".to_string())
                    .or_default()
                    .insert(key.clone(), values.clone());
            }
        }

        activity_log.push("Classifying infrastructure signals".to_string());
        let mut infrastructure_signals = Vec::new();

        if let Some(server_vals) = raw_headers.get("server") {
            for val in server_vals {
                let lower = val.to_lowercase();
                if lower.contains("cloudflare") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "CDN / WAF".to_string(),
                        value: "Cloudflare".to_string(),
                        evidence: format!("Server header explicitly identifies as '{}'", val),
                    });
                } else if lower.contains("nginx") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Web Server".to_string(),
                        value: "Nginx".to_string(),
                        evidence: format!("Server header contains '{}'", val),
                    });
                } else if lower.contains("apache") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Web Server".to_string(),
                        value: "Apache".to_string(),
                        evidence: format!("Server header contains '{}'", val),
                    });
                } else if lower.contains("vercel") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Hosting Platform".to_string(),
                        value: "Vercel".to_string(),
                        evidence: format!("Server header contains '{}'", val),
                    });
                } else if lower.contains("amazon") || lower.contains("aws") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Hosting Platform".to_string(),
                        value: "AWS".to_string(),
                        evidence: format!("Server header contains '{}'", val),
                    });
                } else if lower.contains("iis") {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Web Server".to_string(),
                        value: "Microsoft IIS".to_string(),
                        evidence: format!("Server header contains '{}'", val),
                    });
                } else {
                    infrastructure_signals.push(InfrastructureSignal {
                        signal_type: "Server Identity".to_string(),
                        value: val.clone(),
                        evidence: format!("Server header is exposed as '{}'", val),
                    });
                }
            }
        }

        if let Some(powered_vals) = raw_headers.get("x-powered-by") {
            for val in powered_vals {
                infrastructure_signals.push(InfrastructureSignal {
                    signal_type: "Backend Tech".to_string(),
                    value: val.clone(),
                    evidence: format!("X-Powered-By header is explicitly advertising: '{}'", val),
                });
            }
        }

        if raw_headers.contains_key("cf-ray") {
            infrastructure_signals.push(InfrastructureSignal {
                signal_type: "Edge Proxy".to_string(),
                value: "Cloudflare".to_string(),
                evidence: "CF-Ray header is present in the response".to_string(),
            });
        }

        if raw_headers.contains_key("x-vercel-id") {
            infrastructure_signals.push(InfrastructureSignal {
                signal_type: "Edge Proxy".to_string(),
                value: "Vercel Edge Network".to_string(),
                evidence: "X-Vercel-Id routing header is present".to_string(),
            });
        }

        if raw_headers.contains_key("x-amz-cf-id") {
            infrastructure_signals.push(InfrastructureSignal {
                signal_type: "CDN".to_string(),
                value: "AWS CloudFront".to_string(),
                evidence: "X-Amz-Cf-Id header trace is present".to_string(),
            });
        }

        if let Some(t) = &tx {
            if !infrastructure_signals.is_empty() {
                let _ = t.send(crate::domain::entities::InvestigationEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    event_type: "INFRA_SIGNAL_DETECTED".to_string(),
                    message: format!(
                        "Detected {} explicit infrastructure signals in headers",
                        infrastructure_signals.len()
                    ),
                    payload: Some(
                        serde_json::to_value(&infrastructure_signals)
                            .unwrap_or(serde_json::Value::Null),
                    ),
                });
            }
        }

        activity_log.push("Fetching response body (HTML extraction)".to_string());
        let body_text = resp.text().await.unwrap_or_default();
        let html_lower = body_text.to_lowercase();

        activity_log.push("Fingerprinting CMS and deployments".to_string());
        let mut fingerprints: Vec<InvestigatorFingerprint> = Vec::new();

        let mut add_fp = |name: &str, category: &str, conf: f32, ev: Vec<String>, expl: &str| {
            fingerprints.push(InvestigatorFingerprint {
                name: name.to_string(),
                category: category.to_string(),
                confidence_score: conf.min(100.0),
                evidences: ev,
                explanation: expl.to_string(),
                certainty: Some(if conf >= 80.0 {
                    CertaintyLevel::Certain
                } else if conf >= 50.0 {
                    CertaintyLevel::Likely
                } else {
                    CertaintyLevel::Uncertain
                }),
            });
        };

        // WordPress
        {
            let mut wp_conf = 0.0;
            let mut wp_ev = Vec::new();
            if html_lower.contains("wp-content/") {
                wp_conf += 50.0;
                wp_ev.push("Found 'wp-content/' structural path in HTML body".to_string());
            }
            if html_lower.contains("name=\"generator\" content=\"wordpress") {
                wp_conf += 80.0;
                wp_ev.push("Found explicit WordPress meta generator tag".to_string());
            }
            if raw_headers
                .get("set-cookie")
                .is_some_and(|c| c.iter().any(|v| v.contains("wp-settings")))
            {
                wp_conf += 60.0;
                wp_ev.push("Found 'wp-settings' specific session cookie".to_string());
            }
            if raw_headers
                .get("x-powered-by")
                .is_some_and(|h| h.iter().any(|v| v.to_lowercase().contains("wordpress")))
            {
                wp_conf += 80.0;
                wp_ev.push("Found X-Powered-By WordPress header".to_string());
            }
            if wp_conf > 0.0 {
                add_fp("WordPress", "CMS", wp_conf, wp_ev, "WordPress is the dominant open-source CMS ecosystem. Exploits frequently target outdated third-party plugins in wp-content.");
            }
        }

        // Joomla
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if html_lower.contains("name=\"generator\" content=\"joomla") {
                conf += 90.0;
                ev.push("Found explicit Joomla meta generator tag".to_string());
            }
            if html_lower.contains("/components/com_") || html_lower.contains("/media/jui/") {
                conf += 50.0;
                ev.push(
                    "Found structural Joomla directory paths (components/media) in HTML"
                        .to_string(),
                );
            }
            if conf > 0.0 {
                add_fp(
                    "Joomla!",
                    "CMS",
                    conf,
                    ev,
                    "Joomla is a popular CMS. Security issues often stem from insecure extensions.",
                );
            }
        }

        // Shopify
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if html_lower.contains("cdn.shopify.com") {
                conf += 70.0;
                ev.push("Found asset delivery via cdn.shopify.com".to_string());
            }
            if html_lower.contains("var shopify = shopify")
                || html_lower.contains("shopify.onready")
            {
                conf += 60.0;
                ev.push("Found Shopify global JavaScript object initialization".to_string());
            }
            if let Some(_shop) = raw_headers.get("x-shopid") {
                conf += 100.0;
                ev.push("Explicit Shopify Server Header detected: X-ShopId".to_string());
            }
            if conf > 0.0 {
                add_fp("Shopify", "E-Commerce", conf, ev, "Shopify is a proprietary e-commerce ecosystem. Vulnerabilities are extremely rare in core, but frontend injections are possible.");
            }
        }

        // Drupal
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if html_lower.contains("name=\"generator\" content=\"drupal") {
                conf += 90.0;
                ev.push("Found explicit Drupal meta generator tag".to_string());
            }
            if raw_headers.contains_key("x-drupal-cache")
                || raw_headers
                    .get("x-generator")
                    .is_some_and(|c| c.iter().any(|v| v.to_lowercase().contains("drupal")))
            {
                conf += 80.0;
                ev.push("Found Drupal specific headers".to_string());
            }
            if conf > 0.0 {
                add_fp("Drupal", "CMS", conf, ev, "Drupal is a highly customizable enterprise CMS. Security relies on complex permission handling.");
            }
        }

        // Ghost
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if html_lower.contains("content=\"ghost") {
                conf += 60.0;
                ev.push("Found Ghost meta generator tag or content signature".to_string());
            }
            if raw_headers.contains_key("x-ghost-cache") {
                conf += 90.0;
                ev.push("Found Ghost specific caching headers".to_string());
            }
            if conf > 0.0 {
                add_fp(
                    "Ghost",
                    "CMS",
                    conf,
                    ev,
                    "Ghost is a NodeJS-based headless CMS geared heavily towards publishing.",
                );
            }
        }

        // Vercel (Deployment)
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if raw_headers.contains_key("x-vercel-id") || raw_headers.contains_key("x-vercel-cache")
            {
                conf += 100.0;
                ev.push("Explicit X-Vercel headers strongly identify target".to_string());
            }
            if raw_headers
                .get("server")
                .is_some_and(|c| c.iter().any(|v| v.to_lowercase() == "vercel"))
            {
                conf += 90.0;
                ev.push("Server header broadcasts Vercel".to_string());
            }
            if conf > 0.0 {
                add_fp("Vercel", "Deployment", conf, ev, "Vercel is a frontend-actionable cloud platform utilizing a global edge network.");
            }
        }

        // Netlify
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if raw_headers
                .get("server")
                .is_some_and(|c| c.iter().any(|v| v.to_lowercase() == "netlify"))
            {
                conf += 95.0;
                ev.push("Server header explicitly broadcasts Netlify".to_string());
            }
            if raw_headers.contains_key("x-nf-request-id") {
                conf += 100.0;
                ev.push("X-NF-Request-Id specific header detected".to_string());
            }
            if conf > 0.0 {
                add_fp(
                    "Netlify",
                    "Deployment",
                    conf,
                    ev,
                    "Netlify is a widespread Jamstack deployment delivery network.",
                );
            }
        }

        // Cloudflare (with Fake CDN Detection)
        {
            let has_cf_ray = raw_headers.contains_key("cf-ray");
            let has_cf_server = raw_headers
                .get("server")
                .is_some_and(|c| c.iter().any(|v| v.to_lowercase() == "cloudflare"));
            let leaks_version = raw_headers.contains_key("x-powered-by")
                || raw_headers
                    .get("server")
                    .is_some_and(|c| c.iter().any(|v| v.contains('/')));

            if has_cf_ray && leaks_version && !has_cf_server {
                // Fake Cloudflare — real CF never leaks backend versions
                let mut ev =
                    vec!["CF-Ray header present but server leaks version info".to_string()];
                if let Some(srv) = raw_headers.get("server") {
                    ev.push(format!(
                        "Server header contains version: {}",
                        srv.join(", ")
                    ));
                }
                if raw_headers.contains_key("x-powered-by") {
                    ev.push("X-Powered-By exposed despite alleged CF protection".to_string());
                }
                add_fp(
                    "Fake Cloudflare Headers",
                    "CDN / WAF",
                    85.0,
                    ev,
                    "CF-Ray header is present but the server leaks version information. \
                     Real Cloudflare proxies strip version headers. This may indicate \
                     spoofed CDN headers to appear protected, or a misconfigured reverse proxy.",
                );
            } else if has_cf_ray && has_cf_server && !leaks_version {
                // Genuine Cloudflare — CF-Ray + Server: cloudflare + no version leak
                add_fp("Cloudflare", "CDN / WAF", 100.0,
                    vec![
                        "CF-Ray header detects Cloudflare infrastructure".to_string(),
                        "Server header broadcasts Cloudflare".to_string(),
                        "No version leakage — consistent with real Cloudflare behavior".to_string(),
                    ],
                    "Cloudflare provides DNS layer load-balancing, WAF caching, and edge logic deployment.");
            } else if has_cf_ray {
                // Partial CF signals
                let mut conf = 70.0;
                let mut ev =
                    vec!["CF-Ray header detects possible Cloudflare infrastructure".to_string()];
                if has_cf_server {
                    conf += 20.0;
                    ev.push("Server header broadcasts Cloudflare".to_string());
                }
                add_fp("Cloudflare", "CDN / WAF", conf, ev,
                    "Cloudflare provides DNS layer load-balancing, WAF caching, and edge logic deployment.");
            } else if has_cf_server {
                add_fp("Cloudflare", "CDN / WAF", 80.0,
                    vec!["Server header broadcasts Cloudflare".to_string()],
                    "Cloudflare provides DNS layer load-balancing, WAF caching, and edge logic deployment.");
            }
        }

        // Firebase Hosting
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if raw_headers
                .get("server")
                .is_some_and(|c| c.iter().any(|v| v.to_lowercase() == "firebase"))
            {
                conf += 100.0;
                ev.push("Server header explicitly broadcasts Firebase".to_string());
            }
            if html_lower.contains("__firebase__") {
                conf += 80.0;
                ev.push("Firebase global Javascript injection object isolated".to_string());
            }
            if conf > 0.0 {
                add_fp("Firebase Hosting", "Static Hosting", conf, ev, "Firebase Hosting provides static and dynamic content delivery powered by Google CDN.");
            }
        }

        // GitHub Pages
        {
            let mut conf = 0.0;
            let mut ev = Vec::new();
            if raw_headers
                .get("server")
                .is_some_and(|c| c.iter().any(|v| v.to_lowercase() == "github.com"))
            {
                conf += 100.0;
                ev.push("Server header explicitly broadcasts GitHub.com".to_string());
            }
            if raw_headers.contains_key("x-github-request-id") {
                conf += 100.0;
                ev.push("X-GitHub-Request-Id header detected".to_string());
            }
            if conf > 0.0 {
                add_fp(
                    "GitHub Pages",
                    "Static Hosting",
                    conf,
                    ev,
                    "GitHub Pages serves static compiled Jamstack and Jekyll HTML structures.",
                );
            }
        }

        // Sort footprints by descending confidence
        fingerprints.sort_by(|a, b| {
            b.confidence_score
                .partial_cmp(&a.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(t) = &tx {
            if !fingerprints.is_empty() {
                let _ = t.send(crate::domain::entities::InvestigationEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    event_type: "CMS_FINGERPRINT_MATCHED".to_string(),
                    message: format!("Detected {} deployment fingerprints", fingerprints.len()),
                    payload: Some(
                        serde_json::to_value(&fingerprints).unwrap_or(serde_json::Value::Null),
                    ),
                });
            }
        }

        activity_log.push("Analyzing cache and delivery logistics".to_string());
        let mut delivery_insights: Vec<DeliveryInsight> = Vec::new();

        let mut add_insight = |name: &str, category: &str, conf: f32, ev: &str, expl: &str| {
            delivery_insights.push(DeliveryInsight {
                name: name.to_string(),
                category: category.to_string(),
                confidence_score: conf.min(100.0),
                evidence: ev.to_string(),
                explanation: expl.to_string(),
                certainty: Some(if conf >= 90.0 {
                    CertaintyLevel::Certain
                } else if conf >= 70.0 {
                    CertaintyLevel::Likely
                } else {
                    CertaintyLevel::Uncertain
                }),
            });
        };

        // Cache Strategy
        if let Some(cc) = raw_headers.get("cache-control") {
            let cc_str = cc.join(", ").to_lowercase();
            if cc_str.contains("no-store") || cc_str.contains("no-cache") {
                add_insight(
                    "Strict No-Cache Defended",
                    "Cache Behavior",
                    100.0,
                    &format!("Cache-Control: {}", cc_str),
                    "Target strictly forbids storing payload inside intermediate caches.",
                );
            } else if cc_str.contains("max-age=") && !cc_str.contains("max-age=0") {
                add_insight(
                    "Time-to-Live Caching",
                    "Cache Behavior",
                    90.0,
                    &format!("Cache-Control: {}", cc_str),
                    "Target pushes explicit expiration timers for downstream components to cache.",
                );
            } else if cc_str.contains("must-revalidate") {
                add_insight(
                    "Strict Revalidation Mode",
                    "Cache Behavior",
                    90.0,
                    &format!("Cache-Control: {}", cc_str),
                    "Downstream caches are forced to revalidate assets explicitly with the origin.",
                );
            }
        }

        if raw_headers.contains_key("etag") || raw_headers.contains_key("last-modified") {
            add_insight(
                "Conditional Cache Verification",
                "Cache Behavior",
                80.0,
                "ETag / Last-Modified header found",
                "Server utilizes validator tokens for conditional 304 Not Modified routing.",
            );
        }

        // Edge Delivery
        if let Some(cf_stat) = raw_headers.get("cf-cache-status") {
            let stat = cf_stat.join(", ").to_uppercase();
            if stat.contains("HIT") {
                add_insight("Edge Cache Hit", "Edge/CDN Signal", 100.0, &format!("CF-Cache-Status: {}", stat), "Payload served completely from Cloudflare edge node without querying the origin database.");
            } else if stat.contains("DYNAMIC") || stat.contains("BYPASS") {
                add_insight("Origin Execution Route", "Edge/CDN Signal", 100.0, &format!("CF-Cache-Status: {}", stat), "Cloudflare edge bypassed caching logic and pushed execution directly to the backend origin.");
            } else if stat.contains("MISS") {
                add_insight("Edge Cache Miss", "Edge/CDN Signal", 100.0, &format!("CF-Cache-Status: {}", stat), "Cloudflare edge network missed cache on this route and had to fetch from origin.");
            }
        } else if let Some(x_cache) = raw_headers.get("x-cache") {
            let stat = x_cache.join(", ").to_uppercase();
            if stat.contains("HIT") {
                add_insight("Transit Cache Hit", "Edge/CDN Signal", 95.0, &format!("X-Cache: {}", stat), "Payload was assembled and served from a CDN/Proxy tier and did not hit origin execution.");
            } else {
                add_insight(
                    "Transit Cache Miss",
                    "Edge/CDN Signal",
                    90.0,
                    &format!("X-Cache: {}", stat),
                    "Proxy tier logged a cache miss, resulting in origin interaction.",
                );
            }
        }

        if let Some(age) = raw_headers.get("age") {
            // Check if age > 0 to confirm
            if age.iter().any(|v| v != "0") {
                add_insight("Downstream TTL Decay Detected", "Edge/CDN Signal", 85.0, &format!("Age: {}", age.join(", ")), "Response exhibits an active transit cache Timer-To-Live decay state indicating it was pulled directly from an intermediary node cache, not raw origin execution.");
            }
        }

        // Proxy / Gateway
        if let Some(via) = raw_headers.get("via") {
            add_insight("Transparent Reverse Proxy Routing", "Proxy/Gateway Indicator", 100.0, &format!("Via: {}", via.join(", ")), "Network explicitly tracks requests bouncing cleanly through intermediary proxy routing nodes.");
        }
        if let Some(x_served) = raw_headers.get("x-served-by") {
            add_insight("Explicit Edge Delivery Node", "Proxy/Gateway Indicator", 90.0, &format!("X-Served-By: {}", x_served.join(", ")), "Response includes specific tags tracking identical edge node processing mechanisms (common in Varnish/Fastly clusters).");
        }

        // Sort delivery insights
        delivery_insights.sort_by(|a, b| {
            b.confidence_score
                .partial_cmp(&a.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(t) = &tx {
            if !delivery_insights.is_empty() {
                let _ = t.send(crate::domain::entities::InvestigationEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    event_type: "CACHE_BEHAVIOR_ANALYZED".to_string(),
                    message: format!(
                        "Identified {} routing and cache behaviors",
                        delivery_insights.len()
                    ),
                    payload: Some(
                        serde_json::to_value(&delivery_insights).unwrap_or(serde_json::Value::Null),
                    ),
                });
            }
        }

        activity_log.push("Analyzing security posture and transport configurations".to_string());
        let mut security_insights: Vec<SecurityPostureInsight> = Vec::new();

        let mut add_sec =
            |name: &str, category: &str, status: &str, conf: f32, ev: &str, expl: &str| {
                security_insights.push(SecurityPostureInsight {
                    name: name.to_string(),
                    category: category.to_string(),
                    status: status.to_string(),
                    confidence_score: conf.min(100.0),
                    evidence: ev.to_string(),
                    explanation: expl.to_string(),
                    certainty: Some(if conf >= 90.0 {
                        CertaintyLevel::Certain
                    } else if conf >= 60.0 {
                        CertaintyLevel::Likely
                    } else {
                        CertaintyLevel::Uncertain
                    }),
                });
            };

        // TLS & Transport
        let is_https = final_url.starts_with("https://");
        let was_http = url_str.starts_with("http://") && !url_str.starts_with("https://");

        if is_https {
            if was_http {
                add_sec("Automatic Protocol Upgrade", "TLS & Transport", "Secure", 100.0, "Resolved URL upgraded to HTTPS", "Target explicitly redirected insecure HTTP traffic to the encrypted HTTPS protocol layer.");
            } else {
                add_sec(
                    "Encrypted Transport",
                    "TLS & Transport",
                    "Secure",
                    100.0,
                    "Final URL operates over HTTPS",
                    "Connections to the target are protected by TLS encryption.",
                );
            }
        } else {
            // Localhost/loopback HTTP is expected in dev/test environments — not a real vulnerability
            let is_local = final_url.contains("localhost")
                || final_url.contains("127.0.0.1")
                || final_url.contains("[::1]");
            let status = if is_local {
                "Informational"
            } else {
                "Critical"
            };
            add_sec("Insecure Transport", "TLS & Transport", status, 100.0, "Final URL operates over HTTP", "Traffic is routed over plaintext HTTP, leaving interceptable payloads and credentials entirely exposed.");
        }

        if let Some(hsts) = raw_headers.get("strict-transport-security") {
            let h_val = hsts.join(", ").to_lowercase();
            if h_val.contains("includesubdomains") && h_val.contains("preload") {
                add_sec("Hardened Strict-Transport-Security", "TLS & Transport", "Secure", 100.0, &format!("HSTS: {}", h_val), "HSTS is heavily enforced, shielding all subdomains and preparing for HSTS pinning/preloading.");
            } else {
                add_sec(
                    "Standard Strict-Transport-Security",
                    "TLS & Transport",
                    "Secure",
                    90.0,
                    &format!("HSTS: {}", h_val),
                    "HSTS enforces HTTPS usage specifically on the isolated origin.",
                );
            }
        } else if is_https {
            add_sec("Missing Strict-Transport-Security", "TLS & Transport", "Warning", 90.0, "HSTS header is absent", "Despite HTTPS usage, the lack of HSTS allows potential downgrade attacks like SSL stripping during initial handshakes.");
        }

        // Security Header Interpretation
        if let Some(csp) = raw_headers.get("content-security-policy") {
            let csp_val = csp.join("; ");
            if csp_val.contains("unsafe-inline") || csp_val.contains("unsafe-eval") {
                add_sec("Permissive Content-Security-Policy", "Security Header Interpretation", "Warning", 90.0, &format!("CSP: {}", csp_val), "CSP is implemented but utilizes 'unsafe' allowances which fail to mitigate primary XSS delivery vectors.");
            } else {
                add_sec("Strict Content-Security-Policy", "Security Header Interpretation", "Secure", 95.0, "CSP structure lacks explicit unsafe flags", "Target enforces cross-site scripting (XSS) constraints isolating script executions efficiently.");
            }
        } else {
            add_sec("Missing Content-Security-Policy", "Security Header Interpretation", "Warning", 90.0, "CSP header absent from payload", "Absence of CSP permits total allowance of foreign script execution, heavily increasing XSS impacts if a vulnerability exists.");
        }

        if let Some(xfo) = raw_headers.get("x-frame-options") {
            add_sec("Clickjacking Protection Enforced", "Security Header Interpretation", "Secure", 100.0, &format!("X-Frame-Options: {}", xfo.join(", ")), "X-Frame-Options ensures malicious origins cannot overlay or embed the target inside invisible iframes.");
        } else {
            add_sec("Missing Frame-Options", "Security Header Interpretation", "Warning", 80.0, "X-Frame-Options header missing", "Target does not natively prevent iframe embeddings, opening attack surfaces to Clickjacking.");
        }

        if let Some(xcto) = raw_headers.get("x-content-type-options") {
            if xcto.join("").to_lowercase().contains("nosniff") {
                add_sec("MIME-Sniffing Defeated", "Security Header Interpretation", "Secure", 100.0, "X-Content-Type-Options: nosniff", "The browser is explicitly blocked from re-interpreting files, mitigating mime-confusion exploits.");
            }
        } else {
            add_sec("Permissive MIME-Sniffing", "Security Header Interpretation", "Warning", 85.0, "X-Content-Type-Options missing", "Browsers are permitted to guess asset types, enabling potential execution of concealed payloads masquerading as images/fonts.");
        }

        if let Some(rp) = raw_headers.get("referrer-policy") {
            add_sec("Referrer Constraints Mapped", "Security Header Interpretation", "Informational", 90.0, &format!("Referrer-Policy: {}", rp.join(", ")), "Target explicitly decides how much endpoint routing information is exposed outwards when users click off-site links.");
        }

        if let Some(_pp) = raw_headers.get("permissions-policy") {
            add_sec("Permissions Policy Executed", "Security Header Interpretation", "Secure", 95.0, "Permissions-Policy header detected", "Server explicitly locks down embedded browser APIs (Camera, Microphone, Geolocation).");
        }

        // Infrastructure Exposure
        if let Some(cors) = raw_headers.get("access-control-allow-origin") {
            if cors.join("").contains("*") {
                add_sec("Wildcard CORS Allowance", "Infrastructure Exposure", "Critical", 100.0, "Access-Control-Allow-Origin: *", "Cross-Origin Resource Sharing is entirely unrestricted, allowing any external malicious frontend to read data from this endpoint (if unauthenticated).");
            } else {
                add_sec(
                    "Restricted CORS Profiles",
                    "Infrastructure Exposure",
                    "Secure",
                    90.0,
                    &format!("ACAO limits to explicit domains: {}", cors.join(", ")),
                    "CORS prevents foreign domain access, strictly enforcing API encapsulation.",
                );
            }
        }

        if let Some(srv) = raw_headers.get("server") {
            let val = srv.join(", ");
            let has_version = val.chars().any(|c| c.is_ascii_digit());
            if has_version {
                add_sec("Exact Server Version Exposure", "Infrastructure Exposure", "Warning", 100.0, &format!("Server: {}", val), "Target leaks exact version footprints, directly fueling targeted known-CVE reconnaissance.");
            } else {
                add_sec(
                    "Generic Server Disclosure",
                    "Infrastructure Exposure",
                    "Informational",
                    80.0,
                    &format!("Server: {}", val),
                    "Target broadcasts server brand identity without explicit version increments.",
                );
            }
        }

        if let Some(xpb) = raw_headers.get("x-powered-by") {
            add_sec("Stack Framework Leakage", "Infrastructure Exposure", "Warning", 100.0, &format!("X-Powered-By: {}", xpb.join(", ")), "Backend framework and runtime layer is actively broadcasted to clients unnecessarily.");
        }

        if let Some(xforwarded) = raw_headers.get("x-forwarded-for") {
            add_sec("Origin Routing Leakage", "Infrastructure Exposure", "Warning", 90.0, &format!("X-Forwarded-For: {}", xforwarded.join(", ")), "Proxies have accidentally forwarded routing headers backwards into the response payload, leaking internal network configurations.");
        }

        security_insights.sort_by(|a, b| {
            let rank = |s: &str| match s {
                "Critical" => 4,
                "Warning" => 3,
                "Secure" => 2,
                "Informational" => 1,
                _ => 0,
            };
            rank(&b.status).cmp(&rank(&a.status)).then_with(|| {
                b.confidence_score
                    .partial_cmp(&a.confidence_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        activity_log.push("Investigation complete".to_string());

        if let Some(t) = &tx {
            if !security_insights.is_empty() {
                let _ = t.send(crate::domain::entities::InvestigationEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    event_type: "SECURITY_SIGNAL_EVALUATED".to_string(),
                    message: format!(
                        "Mapped {} absolute security perimeter rules on route",
                        security_insights.len()
                    ),
                    payload: Some(
                        serde_json::to_value(&security_insights).unwrap_or(serde_json::Value::Null),
                    ),
                });
            }
        }

        let investigation_certainty = Some(CertaintyNote {
            level: if !fingerprints.is_empty() || !security_insights.is_empty() {
                CertaintyLevel::Certain
            } else {
                CertaintyLevel::Uncertain
            },
            reason: if fingerprints.is_empty() {
                "Hiçbir platform/CMS parmak izi tespit edilemedi — bu bilgi eksikliğini gösterir"
                    .to_string()
            } else {
                "Analiz başarıyla tamamlandı".to_string()
            },
        });

        Ok(ServerInfo {
            original_target: url_str.to_string(),
            resolved_url: final_url,
            status_code,
            latency_ms,
            raw_headers,
            categorized_headers,
            infrastructure_signals,
            fingerprints,
            delivery_insights,
            security_insights,
            routes_checked: Vec::new(),
            consistency_insights: Vec::new(),
            activity_log,
            investigation_certainty,
        })
    }
}
