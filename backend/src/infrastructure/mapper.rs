use crate::domain::entities::{FormMapping, DetectedForm};
use crate::domain::repositories::FormMapperRepository;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

pub struct HttpFormMapper {
    client: Client,
}

impl HttpFormMapper {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl FormMapperRepository for HttpFormMapper {
    async fn map(&self, url_str: &str) -> Result<FormMapping, String> {
        let resp = self.client.get(url_str).send().await.map_err(|e| e.to_string())?;
        let body = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&body);

        let mut detected_forms = Vec::new();
        let mut login_pages = Vec::new();

        // 1. Detect Forms
        let form_selector = Selector::parse("form").unwrap();
        for form in document.select(&form_selector) {
            let action = form.value().attr("action").unwrap_or("#").to_string();
            let method = form.value().attr("method").unwrap_or("get").to_string();
            
            let mut fields = Vec::new();
            let input_selector = Selector::parse("input, select, textarea").unwrap();
            for input in form.select(&input_selector) {
                let name = input.value().attr("name").unwrap_or("unnamed").to_string();
                let input_type = input.value().attr("type").unwrap_or("text").to_string();
                fields.push(format!("{}({})", name, input_type));
                
                // Detection for potential login
                if name.contains("password") || name.contains("pwd") || input_type == "password" {
                    if !login_pages.contains(&url_str.to_string()) {
                        login_pages.push(url_str.to_string());
                    }
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
