pub struct EndpointClassifier;

impl EndpointClassifier {
    // Determine the base confidence an endpoint has depending on where it was found (0.0 to 1.0)
    pub fn score_confidence(source: &str, method: &str) -> f32 {
        let mut score: f32 = match source {
            "HTML Form" => 0.90,
            "Common Endpoint Brute-force" => 1.0,
            "External JS (Dynamic Import)" => 0.85,
            "External JS (XHR/Fetch)" => 0.85,
            "Inline Script (XHR/Fetch)" => 0.85,
            "Config / Base URL" => 0.70,
            "HTML Link" => 0.60,
            "Hardcoded / Regex Fallback" => 0.40,
            "Data Attribute / Meta" => 0.45,
            _ => 0.40,
        };

        if method != "UNKNOWN" {
            score += 0.10;
        }

        if score > 0.95 { 0.95 } else if score < 0.3 { 0.3 } else { score }
    }

    // Measure auth based on tokens nearby (0.0 to 1.0)
    pub fn assess_auth(content: &str, path: &str) -> f32 {
        let path_lower = path.to_lowercase();
        if path_lower.contains("login") || path_lower.contains("auth") || path_lower.contains("token") {
            // These usually don't need auth to access, they ARE the auth endpoints
            return 0.05;
        }

        let mut prob: f32 = 0.10;
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("authorization") { prob += 0.40; }
        if content_lower.contains("bearer ") { prob += 0.30; }
        if content_lower.contains("jwt") { prob += 0.20; }
        if content_lower.contains("x-api-key") { prob += 0.35; }
        if content_lower.contains("credentials") { prob += 0.25; }
        if content_lower.contains("session") { prob += 0.15; }

        if prob > 1.0 { 1.0 } else { prob }
    }

    pub fn evaluate_auth_likelihood(prob: f32) -> String {
        if prob >= 0.5 {
            "Likely".to_string()
        } else if prob >= 0.2 {
            "Low".to_string()
        } else {
            "None".to_string()
        }
    }

    pub fn guess_param_type(name: &str) -> String {
        let n = name.to_lowercase();
        if n.contains("id") || n.contains("uuid") {
            "id".to_string()
        } else if n.contains("email") || n.contains("mail") {
            "email".to_string()
        } else if n.contains("token") || n.contains("jwt") || n.contains("key") || n.contains("auth") || n.contains("secret") {
            "token".to_string()
        } else if n.contains("password") || n.contains("pwd") {
            "password".to_string()
        } else if n.contains("age") || n.contains("count") || n.contains("limit") || n.contains("offset") || n.contains("page") || n.contains("amount") {
            "number".to_string()
        } else {
            "string".to_string()
        }
    }

    // Advanced false positive filter
    pub fn is_false_positive(path: &str) -> bool {
        let lower = path.to_lowercase();
        let static_exts = [
            ".png", ".jpg", ".jpeg", ".gif", ".svg", ".ico", ".webp", ".avif", ".bmp", ".tiff",
            ".woff", ".woff2", ".ttf", ".eot", ".otf",
            ".css", ".map", ".mp4", ".webm", ".zip", ".tar", ".gz",
            ".pdf", ".docx", ".xlsx", ".webmanifest"
        ];
        
        // Exclude specific static assets
        if static_exts.iter().any(|ext| lower.ends_with(ext) || lower.contains(&format!("{}?", ext))) {
            return true;
        }

        // Exclude generic core web frameworks internals unless explicitly an API path wrapper
        if lower.contains("node_modules") || lower.contains("webpack") || lower.contains("_next/static") {
            return true;
        }

        // Exclude analytics / tracking / external generic services that often pollute results
        let blocked_domains_or_paths = [
            "google-analytics", "analytics.js", "mixpanel", "sentry", "bugsnag",
            "hotjar", "clarity", "facebook.com/tr", "pixel", "telemetry"
        ];
        if blocked_domains_or_paths.iter().any(|b| lower.contains(b)) {
            return true;
        }

        false
    }
}
