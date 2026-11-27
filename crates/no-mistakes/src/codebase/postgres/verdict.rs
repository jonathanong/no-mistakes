//! Proof status for one configured SQL policy obligation at an executor site.
use super::{EmbeddedSqlCall, EmbeddedSqlKind};
mod record;
pub use record::{record_sql_policy_verdicts, SqlBranchJudgment, SqlPolicyVerdictContext};

pub use crate::codebase::postgres_analysis::verdict::{
    SqlEvidenceKind, SqlObligationVerdict, SqlRuleApplication, SqlVerdictStatus,
};

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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
