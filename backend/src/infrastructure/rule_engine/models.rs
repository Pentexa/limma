use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Rule Definition: The shape of a YAML/JSON rule file ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(alias = "severity")]
    pub default_severity: String, // e.g. "critical", "high", "medium", "low", "informational"
    #[serde(default = "default_confidence")]
    pub default_confidence: String, // e.g. "certain", "firm", "tentative"
    pub remediation: Option<String>,
    pub tags: Option<Vec<String>>,
    pub compliance_tags: Option<Vec<String>>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_source")]
    pub source: String, // e.g. "limma-core" or "user-custom"
    #[serde(default = "default_pack")]
    pub pack: String,
    #[serde(default = "default_priority")]
    pub priority: u32, // Higher number = higher priority for dedup
    pub dedup_key: Option<String>,
    pub supersedes: Option<Vec<String>>,
    pub scope: Option<RuleScope>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub condition: RuleConditionNode,
}

fn default_true() -> bool {
    true
}
fn default_priority() -> u32 {
    100
}
fn default_confidence() -> String {
    "tentative".to_string()
}
fn default_version() -> String {
    "1.0.0".to_string()
}
fn default_source() -> String {
    "limma-core".to_string()
}
fn default_pack() -> String {
    "default".to_string()
}

// ── Rule Scope: Pre-filter before Evaluation ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleScope {
    pub required_headers: Option<Vec<String>>,
    pub protocols: Option<Vec<String>>,
    pub content_types: Option<Vec<String>>,
}

// ── Declarative Condition Tree ──

#[derive(Debug, Clone, Serialize)]
pub enum RuleConditionNode {
    // Leaf conditions
    HeaderMissing { header: String },
    HeaderPresent { header: String },
    HeaderValueContains { header: String, value: String },
    HeaderValueMatches { header: String, pattern: String },
    StatusCodeIn { codes: Vec<u16> },
    BodyContains { value: String },
    BodyContainsDecoded { value: String }, // Searches through decoded layers (unicode, base64, url)
    TlsState { is_https: bool },
    ContextFlag { flag: String },

    // Logical combinators
    All(Vec<RuleConditionNode>),
    Any(Vec<RuleConditionNode>),
    Not(Box<RuleConditionNode>),
}

// Custom Deserialize implementation to work around serde_yaml 0.9's broken
// externally-tagged enum handling. Our YAML files use a single-key map format:
//   condition:
//     header_missing:
//       header: "content-security-policy"
impl<'de> Deserialize<'de> for RuleConditionNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, serde_yaml::Value> = HashMap::deserialize(deserializer)?;

        if map.len() != 1 {
            return Err(serde::de::Error::custom(format!(
                "Expected exactly one key in condition node, found {}",
                map.len()
            )));
        }

        let (key, value) = map.into_iter().next().unwrap();

        match key.as_str() {
            "header_missing" => {
                #[derive(Deserialize)]
                struct Inner {
                    header: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::HeaderMissing {
                    header: inner.header,
                })
            }
            "header_present" => {
                #[derive(Deserialize)]
                struct Inner {
                    header: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::HeaderPresent {
                    header: inner.header,
                })
            }
            "header_value_contains" => {
                #[derive(Deserialize)]
                struct Inner {
                    header: String,
                    value: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::HeaderValueContains {
                    header: inner.header,
                    value: inner.value,
                })
            }
            "header_value_matches" => {
                #[derive(Deserialize)]
                struct Inner {
                    header: String,
                    pattern: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::HeaderValueMatches {
                    header: inner.header,
                    pattern: inner.pattern,
                })
            }
            "status_code_in" => {
                #[derive(Deserialize)]
                struct Inner {
                    codes: Vec<u16>,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::StatusCodeIn { codes: inner.codes })
            }
            "body_contains" => {
                #[derive(Deserialize)]
                struct Inner {
                    value: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::BodyContains { value: inner.value })
            }
            "tls_state" => {
                #[derive(Deserialize)]
                struct Inner {
                    is_https: bool,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::TlsState {
                    is_https: inner.is_https,
                })
            }
            "context_flag" => {
                #[derive(Deserialize)]
                struct Inner {
                    flag: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::ContextFlag { flag: inner.flag })
            }
            "body_contains_decoded" => {
                #[derive(Deserialize)]
                struct Inner {
                    value: String,
                }
                let inner: Inner =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::BodyContainsDecoded { value: inner.value })
            }
            "all" => {
                let children: Vec<RuleConditionNode> =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::All(children))
            }
            "any" => {
                let children: Vec<RuleConditionNode> =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::Any(children))
            }
            "not" => {
                let inner: RuleConditionNode =
                    serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(RuleConditionNode::Not(Box::new(inner)))
            }
            other => Err(serde::de::Error::custom(format!(
                "Unknown condition type: '{}'. Valid types: header_missing, header_present, \
                 header_value_contains, header_value_matches, status_code_in, body_contains, \
                 body_contains_decoded, tls_state, context_flag, all, any, not",
                other
            ))),
        }
    }
}

// ── Rule Context: Normalized scan data passed to the evaluator ──

#[derive(Debug, Clone)]
pub struct RuleContext {
    pub url: String,
    pub path: String,
    pub is_login: bool,
    pub is_sensitive: bool,
    pub is_authenticated: bool,
    pub status_code: u16,
    pub headers: HashMap<String, String>, // lowercase keys, joined values
    pub body: Option<String>,
    pub is_https: bool,
}

// ── Localized Message Model ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedMessage {
    pub key: String,
    pub params: HashMap<String, String>,
}

impl LocalizedMessage {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            params: HashMap::new(),
        }
    }
    pub fn with_param(mut self, k: &str, v: &str) -> Self {
        self.params.insert(k.to_string(), v.to_string());
        self
    }
}

// ── Dynamic Rule Finding: Output when a rule matches ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRuleFinding {
    pub rule_id: String,
    pub rule_name: String,
    pub category: String,
    pub severity: String, // Kept to not break compatibility, this is the effective one
    pub effective_severity: String,
    pub default_severity: String,
    pub effective_confidence: String,
    pub default_confidence: String,
    pub calibration_reasons: Vec<LocalizedMessage>,
    pub rule_version: String,
    pub rule_pack: String,
    pub compliance_tags: Vec<String>,
    pub description: LocalizedMessage,
    pub remediation: Option<LocalizedMessage>,
    pub tags: Vec<String>,
    pub matched_evidence: Vec<LocalizedMessage>,
    pub priority: u32,
    pub dedup_key: Option<String>,
    pub evaluation_trace: Option<EvaluationTrace>, // Add structured trace
    pub rule_reputation_score: Option<f64>,
    pub feedback_summary: Option<LocalizedMessage>,
    pub source: String, // e.g. "limma-core"
}

// ── Evaluation Trace: Structured reason for rule execution ──
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationTrace {
    pub condition_type: String,
    pub is_met: bool,
    pub detail: Option<LocalizedMessage>,
    pub children: Option<Vec<EvaluationTrace>>,
}
