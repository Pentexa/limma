use crate::domain::entities::{DetectedTechnology, TechEvidence};
use std::collections::HashMap;
use scraper::{Html, Selector};

// Struct to store parsed HTML contexts so we don't parse multiple times
pub struct ScanContext<'a> {
    pub headers: &'a HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    pub html_body: &'a str,
    pub script_urls: Vec<String>,
    pub meta_tags: HashMap<String, String>,
}

impl<'a> ScanContext<'a> {
    pub fn new(html_body: &'a str, headers: &'a HashMap<String, String>) -> Self {
        let mut script_urls = Vec::new();
        let mut meta_tags = HashMap::new();
        let mut cookies = HashMap::new();
        
        if let Some(cookie_val) = headers.get("set-cookie") {
            // Rough extraction of cookie names
            for part in cookie_val.split(',') {
                if let Some(name_end) = part.find('=') {
                    let name = part[..name_end].trim().to_string();
                    cookies.insert(name, "present".to_string());
                }
            }
        }

        if !html_body.is_empty() {
            let document = Html::parse_document(html_body);
            
            if let Ok(meta_selector) = Selector::parse("meta") {
                for meta in document.select(&meta_selector) {
                    if let Some(name) = meta.value().attr("name") {
                        if let Some(content) = meta.value().attr("content") {
                            meta_tags.insert(name.to_lowercase(), content.to_string());
                        }
                    } else if let Some(property) = meta.value().attr("property") {
                        if let Some(content) = meta.value().attr("content") {
                             meta_tags.insert(property.to_lowercase(), content.to_string());
                        }
                    }
                }
            }

            if let Ok(script_selector) = Selector::parse("script") {
                for script in document.select(&script_selector) {
                    if let Some(src) = script.value().attr("src") {
                        script_urls.push(src.to_string());
                    }
                }
            }
        }

        Self {
            headers,
            cookies,
            html_body,
            script_urls,
            meta_tags,
        }
    }
}

pub enum RuleMatch {
    Header(&'static str, &'static str, f32),          // (Key, Substring, Weight)
    Cookie(&'static str, f32),                        // (Substring in Name, Weight)
    HtmlBody(&'static str, f32),                      // (Substring in HTML, Weight)
    ScriptSrc(&'static str, f32),                     // (Substring in SRC, Weight)
    MetaTag(&'static str, &'static str, f32),         // (Name, Substring in Content, Weight)
}

pub struct TechRule {
    pub name: &'static str,
    pub category: &'static str,
    pub rules: Vec<RuleMatch>,
}

pub struct FingerprintEngine {
    db: Vec<TechRule>,
}

impl FingerprintEngine {
    pub fn new() -> Self {
        // Pre-configure the rule database. Easily modular for Phase 3 DB integration.
        Self {
            db: vec![
                // CMS
                TechRule { name: "WordPress", category: "CMS", rules: vec![
                    RuleMatch::MetaTag("generator", "wordpress", 0.9),
                    RuleMatch::HtmlBody("wp-content/themes", 0.6),
                    RuleMatch::HtmlBody("wp-includes", 0.4),
                    RuleMatch::Cookie("wp-settings", 0.8),
                ]},
                TechRule { name: "Shopify", category: "CMS / E-Commerce", rules: vec![
                    RuleMatch::ScriptSrc("cdn.shopify.com", 0.9),
                    RuleMatch::HtmlBody("Shopify.theme", 0.7),
                    RuleMatch::Cookie("_shopify_s", 0.8),
                ]},
                // Frameworks
                TechRule { name: "Next.js", category: "Frontend Framework", rules: vec![
                    RuleMatch::Header("x-powered-by", "next.js", 0.9),
                    RuleMatch::HtmlBody("id=\"__next\"", 0.8),
                    RuleMatch::ScriptSrc("/_next/static", 0.9),
                ]},
                TechRule { name: "React", category: "UI Library", rules: vec![
                    RuleMatch::HtmlBody("data-reactroot", 0.8),
                    RuleMatch::HtmlBody("react-dom.production", 0.7),
                ]},
                TechRule { name: "Vue.js", category: "Frontend Framework", rules: vec![
                    RuleMatch::HtmlBody("data-v-", 0.5),
                ]},
                TechRule { name: "Laravel", category: "Backend Framework", rules: vec![
                    RuleMatch::Cookie("laravel_session", 0.9),
                    RuleMatch::Cookie("XSRF-TOKEN", 0.2), // common but not definitive
                ]},
                TechRule { name: "Express.js", category: "Backend Framework", rules: vec![
                    RuleMatch::Header("x-powered-by", "express", 0.9),
                ]},
                TechRule { name: "PHP", category: "Language", rules: vec![
                    RuleMatch::Header("x-powered-by", "php", 0.9),
                    RuleMatch::Cookie("PHPSESSID", 0.9),
                ]},
                // WAF & Infrastructure
                TechRule { name: "Cloudflare", category: "CDN / WAF", rules: vec![
                    RuleMatch::Header("server", "cloudflare", 0.95),
                    RuleMatch::Header("cf-ray", "", 0.9),
                    RuleMatch::Cookie("__cfduid", 0.9),
                    RuleMatch::Cookie("cf_clearance", 0.8),
                ]},
                TechRule { name: "Nginx", category: "Web Server", rules: vec![
                    RuleMatch::Header("server", "nginx", 0.9),
                ]},
                TechRule { name: "Apache", category: "Web Server", rules: vec![
                    RuleMatch::Header("server", "apache", 0.9),
                ]},
                TechRule { name: "Vercel", category: "Hosting / PaaS", rules: vec![
                    RuleMatch::Header("server", "vercel", 0.95),
                    RuleMatch::Header("x-vercel-id", "", 0.95),
                ]},
                // Analytics
                TechRule { name: "Google Analytics", category: "Analytics", rules: vec![
                    RuleMatch::ScriptSrc("google-analytics.com/analytics.js", 0.9),
                    RuleMatch::ScriptSrc("googletagmanager.com/gtag/js", 0.8),
                ]},
            ],
        }
    }

    pub fn analyze(&self, ctx: &ScanContext) -> Vec<DetectedTechnology> {
        let mut results = Vec::new();

        let lbody = ctx.html_body.to_lowercase(); // optimize single pass lowercasing

        for rule in &self.db {
            let mut evidences = Vec::new();
            let mut total_confidence = 0.0;
            let mut has_match = false;

            for r in &rule.rules {
                match r {
                    RuleMatch::Header(k, v, w) => {
                        if let Some(val) = ctx.headers.get(*k) {
                            if val.to_lowercase().contains(v) || v.is_empty() {
                                has_match = true;
                                evidences.push(TechEvidence {
                                    source: "header".to_string(),
                                    snippet: format!("{}: {}", k, val),
                                });
                                total_confidence += w;
                            }
                        }
                    },
                    RuleMatch::Cookie(c, w) => {
                        for cookie_name in ctx.cookies.keys() {
                            if cookie_name.to_lowercase().contains(&c.to_lowercase()) {
                                has_match = true;
                                evidences.push(TechEvidence {
                                    source: "cookie".to_string(),
                                    snippet: cookie_name.clone(),
                                });
                                total_confidence += w;
                            }
                        }
                    },
                    RuleMatch::HtmlBody(p, w) => {
                        if lbody.contains(&p.to_lowercase()) {
                            has_match = true;
                            evidences.push(TechEvidence {
                                source: "html_marker".to_string(),
                                snippet: format!("Matches: {}", p),
                            });
                            total_confidence += w;
                        }
                    },
                    RuleMatch::ScriptSrc(s, w) => {
                        for url in &ctx.script_urls {
                            if url.to_lowercase().contains(&s.to_lowercase()) {
                                has_match = true;
                                evidences.push(TechEvidence {
                                    source: "script_url".to_string(),
                                    snippet: url.clone(),
                                });
                                total_confidence += w;
                                break; // one match is enough for this rule
                            }
                        }
                    },
                    RuleMatch::MetaTag(n, c, w) => {
                        if let Some(val) = ctx.meta_tags.get(*n) {
                            if val.to_lowercase().contains(&c.to_lowercase()) {
                                has_match = true;
                                evidences.push(TechEvidence {
                                    source: "meta_tag".to_string(),
                                    snippet: format!("<meta name=\"{}\" content=\"{}\">", n, val),
                                });
                                total_confidence += w;
                            }
                        }
                    },
                }
            }

            if has_match {
                // Re-calculating proper bounded probability merging based on matched evidences:
                let mut merge = 1.0;
                for r in &rule.rules {
                    let matched = match r {
                        RuleMatch::Header(k, v, _) => ctx.headers.get(*k).map_or(false, |val| val.to_lowercase().contains(v) || v.is_empty()),
                        RuleMatch::Cookie(c, _) => ctx.cookies.keys().any(|k| k.to_lowercase().contains(&c.to_lowercase())),
                        RuleMatch::HtmlBody(p, _) => lbody.contains(&p.to_lowercase()),
                        RuleMatch::ScriptSrc(s, _) => ctx.script_urls.iter().any(|u| u.to_lowercase().contains(&s.to_lowercase())),
                        RuleMatch::MetaTag(n, c, _) => ctx.meta_tags.get(*n).map_or(false, |v| v.to_lowercase().contains(&c.to_lowercase())),
                    };
                    if matched {
                        let w = match r {
                            RuleMatch::Header(_, _, w) => w,
                            RuleMatch::Cookie(_, w) => w,
                            RuleMatch::HtmlBody(_, w) => w,
                            RuleMatch::ScriptSrc(_, w) => w,
                            RuleMatch::MetaTag(_, _, w) => w,
                        };
                        merge *= 1.0 - w;
                    }
                }
                
                let combined_confidence = 1.0 - merge;

                results.push(DetectedTechnology {
                    name: rule.name.to_string(),
                    category: rule.category.to_string(),
                    confidence_score: combined_confidence,
                    evidences,
                });
            }
        }

        // Sort by confidence DESC
        results.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}
