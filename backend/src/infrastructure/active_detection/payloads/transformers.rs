use crate::domain::active_vuln::ActiveVulnType;
use std::borrow::Cow;

/// Trait for mutating payloads to bypass WAFs or filters.
pub trait PayloadTransformer: Send + Sync {
    fn name(&self) -> &'static str;
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str>;
    /// Not all transformations apply to all vuln types (e.g. SQL comments only for SQLi)
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool;
}

/// Applies standard URL encoding (e.g. < -> %3C)
pub struct UrlEncoder;
impl PayloadTransformer for UrlEncoder {
    fn name(&self) -> &'static str {
        "url_encode"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        let encoded = urlencoding::encode(payload);
        Cow::Owned(encoded.into_owned())
    }
    fn supports(&self, _vuln_type: &ActiveVulnType) -> bool {
        true
    }
}

/// Applies Double URL encoding (e.g. < -> %253C)
pub struct DoubleUrlEncoder;
impl PayloadTransformer for DoubleUrlEncoder {
    fn name(&self) -> &'static str {
        "double_url_encode"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        let first_pass = urlencoding::encode(payload);
        let second_pass = urlencoding::encode(&first_pass);
        Cow::Owned(second_pass.into_owned())
    }
    fn supports(&self, _vuln_type: &ActiveVulnType) -> bool {
        true
    }
}

/// Mutates case of alphabetic characters (e.g. <script> -> <sCrIpT>)
pub struct CaseMutationTransformer;
impl PayloadTransformer for CaseMutationTransformer {
    fn name(&self) -> &'static str {
        "case_mutation"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        let mut mutated = String::with_capacity(payload.len());
        for (i, c) in payload.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                if i % 2 == 0 {
                    mutated.push(c.to_ascii_uppercase());
                } else {
                    mutated.push(c.to_ascii_lowercase());
                }
            } else {
                mutated.push(c);
            }
        }
        Cow::Owned(mutated)
    }
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool {
        use ActiveVulnType::*;
        matches!(vuln_type, ReflectedXss | StoredXss | DomXss | SqlInjectionUnion | SqlInjectionError)
    }
}

/// Injects SQL comments to break signature matching (e.g. UNION SELECT -> UNION/**/SELECT)
pub struct SqlCommentInjector;
impl PayloadTransformer for SqlCommentInjector {
    fn name(&self) -> &'static str {
        "sql_comment_injector"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        if payload.contains(" ") {
            Cow::Owned(payload.replace(" ", "/**/"))
        } else {
            Cow::Borrowed(payload)
        }
    }
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool {
        use ActiveVulnType::*;
        matches!(vuln_type, SqlInjectionError | SqlInjectionUnion | SqlInjectionBlindTime | SqlInjectionBlindBoolean)
    }
}

/// Injects Null Bytes which can truncate WAF checks or backend C-based APIs
pub struct NullByteInjector;
impl PayloadTransformer for NullByteInjector {
    fn name(&self) -> &'static str {
        "null_byte"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        Cow::Owned(format!("%00{}", payload))
    }
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool {
        use ActiveVulnType::*;
        matches!(vuln_type, LocalFileInclusion | PathTraversal | RemoteFileInclusion | CommandInjection)
    }
}

/// Replaces spaces with tab characters (%09) or other whitespaces.
pub struct WhitespaceObfuscator;
impl PayloadTransformer for WhitespaceObfuscator {
    fn name(&self) -> &'static str {
        "whitespace_obfuscator"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        if payload.contains(" ") {
            Cow::Owned(payload.replace(" ", "%09"))
        } else {
            Cow::Borrowed(payload)
        }
    }
    fn supports(&self, _vuln_type: &ActiveVulnType) -> bool {
        true
    }
}

/// Encodes SQL strings to Hex to bypass quote filters (e.g. admin -> 0x61646d696e)
pub struct HexEncoder;
impl PayloadTransformer for HexEncoder {
    fn name(&self) -> &'static str {
        "hex_encode"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        // Just a simple hex encode wrapper for the entire payload to demonstrate, 
        // in a real engine it would parse literals.
        let hex_encoded: String = payload.bytes().map(|b| format!("{:02x}", b)).collect();
        Cow::Owned(format!("0x{}", hex_encoded))
    }
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool {
        use ActiveVulnType::*;
        matches!(vuln_type, SqlInjectionError | SqlInjectionUnion | SqlInjectionBlindTime | SqlInjectionBlindBoolean | NoSqlInjection)
    }
}

use base64::{engine::general_purpose, Engine as _};

/// Encodes payload to Base64 to bypass signature matching, often used with `echo <b64> | base64 -d | sh`
pub struct Base64Encoder;
impl PayloadTransformer for Base64Encoder {
    fn name(&self) -> &'static str {
        "base64_encode"
    }
    fn transform<'a>(&self, payload: &'a str) -> Cow<'a, str> {
        let b64 = general_purpose::STANDARD.encode(payload);
        // For command injection
        Cow::Owned(format!("echo {} | base64 -d | sh", b64))
    }
    fn supports(&self, vuln_type: &ActiveVulnType) -> bool {
        use ActiveVulnType::*;
        matches!(vuln_type, CommandInjection | CommandInjectionBlind | ServerSideTemplateInjection)
    }
}

pub fn get_all_transformers() -> Vec<Box<dyn PayloadTransformer>> {
    vec![
        Box::new(UrlEncoder),
        Box::new(DoubleUrlEncoder),
        Box::new(CaseMutationTransformer),
        Box::new(SqlCommentInjector),
        Box::new(NullByteInjector),
        Box::new(WhitespaceObfuscator),
        Box::new(HexEncoder),
        Box::new(Base64Encoder),
    ]
}
