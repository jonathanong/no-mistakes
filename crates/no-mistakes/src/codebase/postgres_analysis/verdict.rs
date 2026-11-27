use super::options::EmbeddedSqlOptions;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SqlVerdictStatus {
    Violation,
    ProvenClear,
    Abstain,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SqlEvidenceKind {
    Exact,
    ExhaustiveUnion,
    VerifiedPrefix,
    PolicyExemption,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlRuleApplication {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlObligationVerdict {
    pub profile: EmbeddedSqlOptions,
    pub rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<SqlRuleApplication>,
    pub subcheck: String,
    pub file: String,
    pub span: (u32, u32),
    pub status: SqlVerdictStatus,
    pub advisory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<SqlEvidenceKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_prefix: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub helper_lines: Vec<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub residual_reasons: Vec<String>,
}
