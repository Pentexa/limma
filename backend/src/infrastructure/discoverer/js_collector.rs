use regex::Regex;

#[derive(Debug)]

pub struct JsFileAnalysis {
    pub is_minified: bool,
    pub has_sourcemap: bool,
    pub potential_chunks: Vec<String>,
}

pub struct JsCollector;

impl JsCollector {
    pub fn analyze_metadata(js_content: &str) -> JsFileAnalysis {
        let is_minified = Self::check_minified(js_content);
        let has_sourcemap = js_content.contains("//# sourceMappingURL=");
        let potential_chunks = Self::find_dynamic_imports(js_content);

        JsFileAnalysis {
            is_minified,
            has_sourcemap,
            potential_chunks,
        }
    }

    fn check_minified(content: &str) -> bool {
        // Simple heuristic: very few newlines compared to file length
        let newlines = content.bytes().filter(|&b| b == b'\n').count();
        if content.len() > 1024 * 50 {
            // Over 50kb
            newlines < 15
        } else {
            // Smaller files: average line length
            if newlines == 0 {
                return true;
            }
            (content.len() / newlines) > 200
        }
    }

    fn find_dynamic_imports(content: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        // Webpack/Vite chunk loading regex heuristics
        // E.g: __webpack_require__.e("chunk-123")
        // E.g: import("./lazy-component")
        if let Ok(re) = Regex::new(r#"(?i)(?:import|require\.e)\s*\(\s*['"]([^'"]+)['"]\s*\)"#) {
            for cap in re.captures_iter(content) {
                if let Some(chunk) = cap.get(1) {
                    let chunk_str = chunk.as_str();
                    if chunk_str.ends_with(".js") || chunk_str.contains(".chunk.") {
                        chunks.push(chunk_str.to_string());
                    }
                }
            }
        }
        chunks
    }
}
