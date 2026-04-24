use scraper::{Html, Selector};
use url::Url;

pub struct HtmlAnalyzer;

#[derive(Debug)]
pub struct HtmlAnalysisRes {
    pub forms: Vec<(String, String, Vec<String>)>, // action, method, params
    pub links: Vec<String>,
    pub inline_scripts: Vec<String>,
    pub external_js_src: Vec<String>,
    pub data_endpoints: Vec<String>,
    pub base_href: Option<String>,
    pub spa_states: Vec<String>, // the raw string of the <script id="__NEXT_DATA__"> or similar
}

impl HtmlAnalyzer {
    pub fn parse(html_body: &str, _base_url: &Url) -> HtmlAnalysisRes {
        let document = Html::parse_document(html_body);
        let mut res = HtmlAnalysisRes {
            forms: Vec::new(),
            links: Vec::new(),
            inline_scripts: Vec::new(),
            external_js_src: Vec::new(),
            data_endpoints: Vec::new(),
            base_href: None,
            spa_states: Vec::new(),
        };

        // 1. Base Href
        if let Ok(sel) = Selector::parse("base[href]") {
            for node in document.select(&sel) {
                if let Some(href) = node.value().attr("href") {
                    res.base_href = Some(href.to_string());
                }
            }
        }

        // 2. Forms
        if let Ok(form_sel) = Selector::parse("form") {
            for form in document.select(&form_sel) {
                if let Some(action) = form.value().attr("action") {
                    if action == "#" || action.is_empty() {
                        continue;
                    }
                    let method = form.value().attr("method").unwrap_or("GET").to_uppercase();

                    let mut params = Vec::new();
                    if let Ok(input_sel) = Selector::parse("input, select, textarea") {
                        for input in form.select(&input_sel) {
                            if let Some(name) = input.value().attr("name") {
                                params.push(name.to_string());
                            }
                        }
                    }
                    res.forms.push((action.to_string(), method, params));
                }
            }
        }

        // 3. Links (A tags)
        if let Ok(a_sel) = Selector::parse("a[href]") {
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    if href.starts_with("#") || href.starts_with("javascript:") {
                        continue;
                    }
                    res.links.push(href.to_string());
                }
            }
        }

        // 4. Scripts (External JS src + Inline texts)
        if let Ok(script_sel) = Selector::parse("script") {
            for script in document.select(&script_sel) {
                if let Some(src) = script.value().attr("src") {
                    res.external_js_src.push(src.to_string());
                } else {
                    let text = script.text().collect::<Vec<_>>().join(" ");
                    if !text.trim().is_empty() {
                        // Check if it's SPA hydration data
                        if text.contains("\"props\":")
                            || text.contains("__NEXT_DATA__")
                            || text.contains("window.__PRELOADED_STATE__")
                        {
                            res.spa_states.push(text);
                        } else {
                            res.inline_scripts.push(text);
                        }
                    }
                }
            }
        }

        // 5. Data-* and meta/config traces
        if let Ok(all_sel) = Selector::parse("*") {
            for node in document.select(&all_sel) {
                for (attr, val) in node.value().attrs() {
                    if attr.starts_with("data-") {
                        // Very rough heuristic to see if the data attribute holds an endpoint
                        if val.starts_with('/')
                            || val.starts_with("http")
                            || val.contains(".php")
                            || val.contains(".do")
                        {
                            res.data_endpoints.push(val.to_string());
                        }
                    }
                    // Extract common meta config
                    if node.value().name() == "meta" {
                        let name_or_prop = node
                            .value()
                            .attr("name")
                            .or(node.value().attr("property"))
                            .unwrap_or("");
                        if name_or_prop.contains("api-base") || name_or_prop.contains("endpoint") {
                            if let Some(content) = node.value().attr("content") {
                                res.data_endpoints.push(content.to_string());
                            }
                        }
                    }
                }
            }
        }

        res
    }
}
