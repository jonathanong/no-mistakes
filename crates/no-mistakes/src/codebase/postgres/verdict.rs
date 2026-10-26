//! Proof status for one configured SQL policy obligation at an executor site.
use super::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlOptions};
use serde::Serialize;
mod record;
pub use record::{record_sql_policy_verdicts, SqlBranchJudgment, SqlPolicyVerdictContext};

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

pub(crate) fn sql_evidence_for_call(
    call: &EmbeddedSqlCall,
    status: SqlVerdictStatus,
) -> Option<SqlEvidenceKind> {
    if status == SqlVerdictStatus::Abstain {
        None
    } else if call.variants_truncated() {
        Some(SqlEvidenceKind::VerifiedPrefix)
    } else if !call.variants.is_empty() {
        Some(SqlEvidenceKind::ExhaustiveUnion)
    } else if call.kind == EmbeddedSqlKind::Dynamic {
        Some(SqlEvidenceKind::VerifiedPrefix)
    } else {
        Some(SqlEvidenceKind::Exact)
    }
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

/// Combine every permitted branch of one obligation. The caller must supply
/// `exhaustive = false` for `Possible`, a cap overflow, or an unproved branch.
/// A positive finding survives an open branch only when its evaluator proved
/// the violation independently of the missing SQL.
pub fn combine_sql_verdicts(
    branches: &[SqlVerdictStatus],
    exhaustive: bool,
    positive_survives_unknown: bool,
) -> SqlVerdictStatus {
    let violation = branches.contains(&SqlVerdictStatus::Violation);
    if violation && (exhaustive || positive_survives_unknown) {
        return SqlVerdictStatus::Violation;
    }
    if exhaustive
        && !branches.is_empty()
        && branches
            .iter()
            .all(|branch| *branch == SqlVerdictStatus::ProvenClear)
    {
        return SqlVerdictStatus::ProvenClear;
    }
    SqlVerdictStatus::Abstain
}

#[cfg(test)]
mod tests;
