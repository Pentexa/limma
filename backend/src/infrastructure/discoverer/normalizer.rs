use crate::domain::entities::{EndpointDetail, Evidence, ParamDetail};
use std::collections::HashMap;
use url::Url;
use crate::infrastructure::discoverer::classifier::EndpointClassifier;

pub struct PathNormalizer;

impl PathNormalizer {
    pub fn resolve_and_merge(
        endpoints_map: &mut HashMap<String, EndpointDetail>,
        raw_path: &str,
        base_url: &Url,
        source_type: &str,
        snippet: &str,
        predicted_method: &str,
        input_fields: Vec<String>, // form fields or raw JS objects
        auth_prob: f32,
        mut conf: f32,
        reason: &str,
        line_number: Option<usize>,
    ) {
        // Fallback UNKNOWN method to GET and lower its confidence slightly
        let method = if predicted_method == "UNKNOWN" {
            conf = conf * 0.85; // Penalty for fallback
            if conf < 0.3 { conf = 0.3; } // Maintain lower bound limit
            "GET"
        } else {
            predicted_method
        };

        // Resolve absolute URL
        let resolved_url = match base_url.join(raw_path) {
            Ok(u) => u,
            Err(_) => return, // Ignore entirely malformed paths
        };

        let host = resolved_url.host_str().unwrap_or("");
        let base_host = base_url.host_str().unwrap_or("");
        
        if host != base_host && !host.contains("api.") && !host.contains("graphql") && !host.contains("firebase") {
            return; 
        }

        let mut param_details: Vec<ParamDetail> = Vec::new();

        // 1. Process explicit Path variables (e.g. from template literals mapping [VAR])
        let mut final_path = format!("{}://{}{}", 
            resolved_url.scheme(),
            resolved_url.host_str().unwrap_or(""),
            resolved_url.path()
        );

        if final_path.contains("[VAR]") {
            param_details.push(ParamDetail {
                name: "id_or_slug".to_string(), // Abstract name
                param_type: "path".to_string(),
                data_type: "id".to_string(),
            });
            final_path = final_path.replace("[VAR]", ":id_or_slug"); // Express-like route formatting
        }

        // 2. Query Parameters embedded directly into the URL
        for (key, _val) in resolved_url.query_pairs() {
            param_details.push(ParamDetail {
                name: key.to_string(),
                param_type: "query".to_string(),
                data_type: EndpointClassifier::guess_param_type(&key),
            });
        }

        // 3. Extracted Input parameters (from HTML <form> or JS inputs)
        for input in input_fields {
            // Check if form was POST/PUT -> Body type, else Query
            let p_type = if method == "POST" || method == "PUT" || method == "PATCH" {
                "body"
            } else {
                "query"
            };
            
            // Prevent duplicated params (e.g. ?name=x and <input name="name"> mapping)
            if !param_details.iter().any(|pd| pd.name == input) {
                param_details.push(ParamDetail {
                    name: input.clone(),
                    param_type: p_type.to_string(),
                    data_type: EndpointClassifier::guess_param_type(&input),
                });
            }
        }

        let canonical_path = final_path;

        let evidence = Evidence {
            source_type: source_type.to_string(),
            snippet: snippet.trim().to_string(),
            reason: reason.to_string(),
            line_number,
        };

        // Deduplication & Synthesis Merge
        let is_novel = !endpoints_map.contains_key(&canonical_path);
        let entry = endpoints_map.entry(canonical_path.clone()).or_insert(EndpointDetail {
            path: canonical_path.clone(), 
            method_prediction: method.to_string(),
            parameters: Vec::new(),
            auth_probability: auth_prob,
            auth_likelihood: EndpointClassifier::evaluate_auth_likelihood(auth_prob),
            confidence_score: conf,
            evidences: Vec::new(),
            runtime_verification: None,
            certainty: None,
        });

        // Push unique evidence
        let mut is_new_evidence = false;
        if !entry.evidences.iter().any(|e| e.snippet == evidence.snippet) {
            entry.evidences.push(evidence);
            is_new_evidence = true;
        }

        if !is_novel && is_new_evidence {
            // Compound confidence formula: 1 - (1 - c1) * (1 - c2)
            let mut combined = 1.0 - ((1.0 - entry.confidence_score) * (1.0 - conf));
            if combined > 0.95 { combined = 0.95; }
            entry.confidence_score = combined;
        } else if is_novel {
            entry.confidence_score = conf; // Use raw conf initially
        } else if conf > entry.confidence_score {
            entry.confidence_score = conf; // Safe fallback if evidence was duplicate but confidence was maybe higher for some reason
        }

        // Method Override handling 
        if entry.method_prediction == "GET" && method != "GET" && conf > entry.confidence_score {
            entry.method_prediction = method.to_string(); // POST/PUT is more explicit
        }

        // Merge parameters intelligently without exact duplicates
        for p in param_details {
            if !entry.parameters.iter().any(|existing| existing.name == p.name) {
                entry.parameters.push(p);
            }
        }

        if auth_prob > entry.auth_probability {
            entry.auth_probability = auth_prob;
            entry.auth_likelihood = EndpointClassifier::evaluate_auth_likelihood(auth_prob);
        }
    }
}
