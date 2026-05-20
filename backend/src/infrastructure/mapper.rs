use crate::domain::entities::{DetectedForm, FormMapping};
use crate::domain::repositories::FormMapperRepository;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

pub struct HttpFormMapper {}

impl HttpFormMapper {
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
impl FormMapperRepository for HttpFormMapper {
    async fn map(
        &self,
        url_str: &str,
        profile: &crate::domain::engine_config::EngineConfig,
    ) -> Result<FormMapping, String> {
        let client = self.build_client(profile);
        let resp = client
            .get(url_str)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&body);

        let mut detected_forms = Vec::new();
        let mut login_pages = Vec::new();

        // 1. Detect Forms
        let form_selector = Selector::parse("form").expect("valid CSS selector");
        for form in document.select(&form_selector) {
            let action = form.value().attr("action").unwrap_or("#").to_string();
            let method = form.value().attr("method").unwrap_or("get").to_string();

            let mut fields = Vec::new();
            let input_selector =
                Selector::parse("input, select, textarea").expect("valid CSS selector");
            for input in form.select(&input_selector) {
                let name = input.value().attr("name").unwrap_or("unnamed").to_string();
                let input_type = input.value().attr("type").unwrap_or("text").to_string();
                
                // Extract hidden inputs logic currently defaults to true since it's not in EngineConfig
                // We'll leave it as true for now.
                
                fields.push(format!("{}({})", name, input_type));

                // Detection for potential login
                if (name.contains("password") || name.contains("pwd") || input_type == "password")
                    && !login_pages.contains(&url_str.to_string())
                {
                    login_pages.push(url_str.to_string());
                }
            }

            detected_forms.push(DetectedForm {
                action,
                method,
                fields,
            });
        }

        // 2. Keyword check in URL for login pages (if not found by forms)
        let login_keywords = vec!["login", "signin", "auth", "account", "giris"];
        if login_pages.is_empty() {
            for kw in login_keywords {
                if body.to_lowercase().contains(kw) || url_str.to_lowercase().contains(kw) {
                    // This is a weak detection but identifies pages that might have JS-based logins
                    login_pages.push(format!("Potential: {}", kw));
                }
            }
        }

        Ok(FormMapping {
            url: url_str.to_string(),
            detected_forms,
            login_pages_found: login_pages,
        })
    }
}
