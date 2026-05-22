pub mod cmdi_payloads;
pub mod deser_payloads;
pub mod graphql_payloads;
pub mod idor_payloads;
pub mod jwt_payloads;
pub mod lfi_payloads;
pub mod nosqli_payloads;
pub mod redirect_payloads;
pub mod sqli_payloads;
pub mod ssrf_payloads;
pub mod ssti_payloads;
pub mod transformers;
pub mod xss_payloads;
pub mod xxe_payloads;

use crate::domain::active_vuln::{ActiveVulnType, PayloadDefinition};
use std::collections::HashMap;
use transformers::{get_all_transformers, PayloadTransformer};

/// Central payload database holding all attack payloads indexed by vulnerability type.
pub struct PayloadDatabase {
    payloads: HashMap<ActiveVulnType, Vec<PayloadDefinition>>,
    transformers: Vec<Box<dyn PayloadTransformer>>,
}

impl PayloadDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            payloads: HashMap::new(),
            transformers: get_all_transformers(),
        };
        db.load_all_payloads();
        db
    }

    fn load_all_payloads(&mut self) {
        self.payloads.insert(
            ActiveVulnType::ReflectedXss,
            xss_payloads::get_reflected_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::StoredXss,
            xss_payloads::get_stored_payloads(),
        );
        self.payloads
            .insert(ActiveVulnType::DomXss, xss_payloads::get_dom_payloads());
        self.payloads.insert(
            ActiveVulnType::SqlInjectionError,
            sqli_payloads::get_error_based(),
        );
        self.payloads.insert(
            ActiveVulnType::SqlInjectionUnion,
            sqli_payloads::get_union_based(),
        );
        self.payloads.insert(
            ActiveVulnType::SqlInjectionBlindTime,
            sqli_payloads::get_time_based(),
        );
        self.payloads.insert(
            ActiveVulnType::SqlInjectionBlindBoolean,
            sqli_payloads::get_boolean_based(),
        );
        self.payloads.insert(
            ActiveVulnType::CommandInjection,
            cmdi_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::CommandInjectionBlind,
            cmdi_payloads::get_blind_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::LocalFileInclusion,
            lfi_payloads::get_lfi_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::RemoteFileInclusion,
            lfi_payloads::get_rfi_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::PathTraversal,
            lfi_payloads::get_traversal_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::ServerSideRequestForgery,
            ssrf_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::XmlExternalEntity,
            xxe_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::OpenRedirect,
            redirect_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::JwtNoneAlgorithm,
            jwt_payloads::get_none_alg_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::JwtWeakSecret,
            jwt_payloads::get_weak_secret_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::NoSqlInjection,
            nosqli_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::ServerSideTemplateInjection,
            ssti_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::GraphqlIntrospectionEnabled,
            graphql_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::GraphqlAbuse,
            graphql_payloads::get_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::InsecureDeserializationJava,
            deser_payloads::get_java_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::InsecureDeserializationPhp,
            deser_payloads::get_php_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::InsecureDeserializationPython,
            deser_payloads::get_python_payloads(),
        );
        self.payloads.insert(
            ActiveVulnType::InsecureDirectObjectReference,
            idor_payloads::get_payloads(),
        );
    }

    pub fn get_payloads(
        &self,
        vuln_type: ActiveVulnType,
        safe_mode: bool,
    ) -> Vec<PayloadDefinition> {
        self.payloads
            .get(&vuln_type)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| !safe_mode || p.safe_for_production)
            .collect()
    }

    pub fn get_payloads_with_bypass(
        &self,
        vuln_type: ActiveVulnType,
        safe_mode: bool,
    ) -> Vec<PayloadDefinition> {
        let base_payloads = self.get_payloads(vuln_type, safe_mode);
        let mut all_payloads = base_payloads.clone();

        for base in base_payloads {
            for transformer in &self.transformers {
                if transformer.supports(&vuln_type) {
                    let transformed_str = transformer.transform(&base.payload);
                    // Avoid duplicating identical transformations
                    if transformed_str.as_ref() != base.payload {
                        let mut mutated = base.clone();
                        mutated.id = format!("{}_waf_{}", mutated.id, transformer.name());
                        mutated.payload = transformed_str.into_owned();
                        mutated.description = format!(
                            "{} (WAF Bypass: {})",
                            mutated.description,
                            transformer.name()
                        );
                        all_payloads.push(mutated);
                    }
                }
            }
        }
        all_payloads
    }

    pub fn get_all_payloads_for_types(
        &self,
        types: &[ActiveVulnType],
        safe_mode: bool,
    ) -> Vec<(ActiveVulnType, PayloadDefinition)> {
        let mut result = Vec::new();
        for vt in types {
            for payload in self.get_payloads(*vt, safe_mode) {
                result.push((*vt, payload));
            }
        }
        result
    }

    
    pub fn total_payload_count(&self) -> usize {
        self.payloads.values().map(|v| v.len()).sum()
    }
}
