use crate::domain::entities::*;

pub struct GraphqlAnalyzer {
    client: reqwest::Client,
}

impl Default for GraphqlAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphqlAnalyzer {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn detect_graphql_abuse(
        &self,
        target_url: &str,
    ) -> Result<Vec<RawBlindFinding>, String> {
        let mut findings = Vec::new();

        // 1. Common GraphQL endpoints to check if target_url doesn't already point to one
        let mut endpoints = vec![target_url.to_string()];
        
        if !target_url.to_lowercase().contains("graphql") {
            let base_url = target_url.trim_end_matches('/');
            endpoints.extend(vec![
                format!("{}/graphql", base_url),
                format!("{}/api/graphql", base_url),
                format!("{}/v1/graphql", base_url),
            ]);
        }

        let introspection_query = r#"{"query": "\n    query IntrospectionQuery {\n      __schema {\n        queryType { name }\n        mutationType { name }\n        subscriptionType { name }\n        types {\n          ...FullType\n        }\n        directives {\n          name\n          description\n          locations\n          args {\n            ...InputValue\n          }\n        }\n      }\n    }\n\n    fragment FullType on __Type {\n      kind\n      name\n      description\n      fields(includeDeprecated: true) {\n        name\n        description\n        args {\n          ...InputValue\n        }\n        type {\n          ...TypeRef\n        }\n        isDeprecated\n        deprecationReason\n      }\n      inputFields {\n        ...InputValue\n      }\n      interfaces {\n        ...TypeRef\n      }\n      enumValues(includeDeprecated: true) {\n        name\n        description\n        isDeprecated\n        deprecationReason\n      }\n      possibleTypes {\n        ...TypeRef\n      }\n    }\n\n    fragment InputValue on __InputValue {\n      name\n      description\n      type { ...TypeRef }\n      defaultValue\n    }\n\n    fragment TypeRef on __Type {\n      kind\n      name\n      ofType {\n        kind\n        name\n        ofType {\n          kind\n          name\n          ofType {\n            kind\n            name\n            ofType {\n              kind\n              name\n              ofType {\n                kind\n                name\n                ofType {\n                  kind\n                  name\n                  ofType {\n                    kind\n                    name\n                  }\n                }\n              }\n            }\n          }\n        }\n      }\n    }\n  "}"#;

        for ep in endpoints {
            let res = self.client
                .post(&ep)
                .header("Content-Type", "application/json")
                .body(introspection_query.to_string())
                .send()
                .await;

            if let Ok(response) = res {
                if response.status().is_success() {
                    if let Ok(body) = response.text().await {
                        // FP=0 Check: Must strictly contain __schema JSON structure
                        if body.contains("\"__schema\"") && body.contains("\"queryType\"") && body.contains("\"types\"") {
                            findings.push(RawBlindFinding {
                                target_url: ep.clone(),
                                parameter: Some("GraphQL Endpoint".to_string()),
                                vulnerability_type: BlindVulnType::GraphqlAbuse,
                                detection_method: BlindDetectionMethod::DifferentialAnalysis,
                                payload_used: "IntrospectionQuery".to_string(),
                                raw_confidence: 1.0, // 100% confidence due to strict JSON matching
                                evidence: BlindEvidence {
                                    dom_snapshot: None,
                                    timing_comparison: None,
                                    callback_received: None,
                                    payload_hash: format!("graphql_intro_{}", ep),
                                },
                            });
                            break; // Stop at first successful introspection endpoint
                        }
                    }
                }
            }
        }

        Ok(findings)
    }
}
