use crate::domain::entities::*;

pub struct RuleEngine {
    pub rules: Vec<AuditRule>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    pub fn evaluate(&self, findings: &[SecurityAuditFinding]) -> Vec<RuleMatchResult> {
        let mut results = Vec::new();

        for finding in findings {
            for rule in &self.rules {
                if let Some(match_result) = self.evaluate_rule(finding, rule) {
                    results.push(match_result);
                }
            }
        }

        results
    }

    fn evaluate_rule(
        &self,
        finding: &SecurityAuditFinding,
        rule: &AuditRule,
    ) -> Option<RuleMatchResult> {
        let mut evaluations = Vec::new();
        let mut mandatory_met_count = 0;
        let mut total_mandatory = 0;
        let mut optional_met_count = 0;
        let mut total_optional = 0;

        for condition in &rule.conditions {
            if condition.is_mandatory {
                total_mandatory += 1;
            } else {
                total_optional += 1;
            }

            let is_met = match condition.condition_type {
                RuleConditionType::CategoryIs => {
                    format!("{:?}", finding.category).to_lowercase()
                        == condition.expected_value.to_lowercase()
                }
                RuleConditionType::SeverityMin => {
                    let finding_sev_val = match finding.severity {
                        SeverityLevel::Critical => 4,
                        SeverityLevel::High => 3,
                        SeverityLevel::Medium => 2,
                        SeverityLevel::Low => 1,
                        SeverityLevel::Informational => 0,
                    };
                    let exp_sev_val = match condition.expected_value.to_lowercase().as_str() {
                        "critical" => 4,
                        "high" => 3,
                        "medium" => 2,
                        "low" => 1,
                        "informational" => 0,
                        _ => 0,
                    };
                    finding_sev_val >= exp_sev_val
                }
                RuleConditionType::ConfidenceMin => {
                    let finding_conf_val = match finding.confidence {
                        ConfidenceLevel::Certain => 4,
                        ConfidenceLevel::High => 3,
                        ConfidenceLevel::Firm => 2,
                        ConfidenceLevel::Medium => 1,
                        ConfidenceLevel::Tentative => 0,
                        ConfidenceLevel::Low => -1,
                    };
                    let exp_conf_val = match condition.expected_value.to_lowercase().as_str() {
                        "certain" => 3,
                        "firm" => 2,
                        "tentative" => 1,
                        "low" => 0,
                        _ => 0,
                    };
                    finding_conf_val >= exp_conf_val
                }
                RuleConditionType::SummaryContains => finding
                    .summary
                    .to_lowercase()
                    .contains(&condition.expected_value.to_lowercase()),
                RuleConditionType::HasEvidence => finding.evidence.iter().any(|e| {
                    e.raw_data
                        .to_lowercase()
                        .contains(&condition.expected_value.to_lowercase())
                }),
                RuleConditionType::ProtocolIs => {
                    if let Some(ref p) = finding.protocol {
                        p.to_lowercase() == condition.expected_value.to_lowercase()
                    } else {
                        false
                    }
                }
                RuleConditionType::MethodIs => {
                    if let Some(ref m) = finding.method {
                        m.to_lowercase() == condition.expected_value.to_lowercase()
                    } else {
                        false
                    }
                }
                RuleConditionType::SourceModuleIs => {
                    format!("{:?}", finding.source_module).to_lowercase()
                        == condition.expected_value.to_lowercase()
                }
            };

            if is_met {
                if condition.is_mandatory {
                    mandatory_met_count += 1;
                } else {
                    optional_met_count += 1;
                }
            }

            evaluations.push(AuditConditionEvaluation {
                condition: condition.clone(),
                is_met,
                detail: if is_met {
                    "Condition satisfied.".into()
                } else {
                    "Condition failed.".into()
                },
            });
        }

        if total_mandatory > 0 && mandatory_met_count < total_mandatory {
            return None;
        }

        let outcome = if total_optional > 0 && optional_met_count < total_optional {
            RuleOutcome::PartiallyMatched
        } else {
            RuleOutcome::Matched
        };

        if outcome == RuleOutcome::Matched || outcome == RuleOutcome::PartiallyMatched {
            Some(RuleMatchResult {
                rule_id: rule.id.clone(),
                rule_title: rule.title.clone(),
                outcome,
                finding_id: finding.id.clone(),
                summary: rule.title.clone(),
                evaluations,
            })
        } else {
            None
        }
    }

    fn default_rules() -> Vec<AuditRule> {
        vec![
            AuditRule {
                id: "RULE-001".into(),
                title: "Missing CSP Detected".into(),
                description: "The application fails to define a CSP, leading to potential XSS vulnerabilities.".into(),
                category_mapping: FindingCategory::SecurityMisconfiguration,
                default_severity: SeverityLevel::Medium,
                conditions: vec![
                    RuleCondition {
                        condition_type: RuleConditionType::CategoryIs,
                        expected_value: "SecurityMisconfiguration".into(),
                        is_mandatory: true,
                    },
                    RuleCondition {
                        condition_type: RuleConditionType::SummaryContains,
                        expected_value: "content-security-policy".into(),
                        is_mandatory: true,
                    }
                ]
            },
            AuditRule {
                id: "RULE-002".into(),
                title: "Exposed Sensitive Endpoint".into(),
                description: "A sensitive administrative route lacks explicit authentication protocols.".into(),
                category_mapping: FindingCategory::AuthenticationBypass,
                default_severity: SeverityLevel::High,
                conditions: vec![
                    RuleCondition {
                        condition_type: RuleConditionType::CategoryIs,
                        expected_value: "AuthenticationBypass".into(),
                        is_mandatory: true,
                    },
                    RuleCondition {
                        condition_type: RuleConditionType::SummaryContains,
                        expected_value: "admin".into(),
                        is_mandatory: false,
                    },
                    RuleCondition {
                        condition_type: RuleConditionType::ConfidenceMin,
                        expected_value: "Tentative".into(),
                        is_mandatory: true,
                    }
                ]
            },
            AuditRule {
                id: "RULE-003".into(),
                title: "Harmful Server Banner Disclosed".into(),
                description: "The server explicitly advertises its specific software version, assisting targeted exploitation.".into(),
                category_mapping: FindingCategory::InformationDisclosure,
                default_severity: SeverityLevel::Low,
                conditions: vec![
                    RuleCondition {
                        condition_type: RuleConditionType::SourceModuleIs,
                        expected_value: "ServerInvestigator".into(),
                        is_mandatory: true,
                    },
                    RuleCondition {
                        condition_type: RuleConditionType::CategoryIs,
                        expected_value: "InformationDisclosure".into(),
                        is_mandatory: true,
                    }
                ]
            }
        ]
    }
}
