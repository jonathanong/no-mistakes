use super::{combine_sql_verdicts, SqlEvidenceKind, SqlObligationVerdict, SqlVerdictStatus};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions,
    SqlStatementFileFacts,
};
use std::borrow::Cow;

/// A branch judgment made by the owning rule's actual policy evaluator.
pub struct SqlBranchJudgment {
    pub status: SqlVerdictStatus,
    pub survives_unknown: bool,
    pub reason: Option<&'static str>,
}

impl SqlBranchJudgment {
    pub fn clear() -> Self {
        Self {
            status: SqlVerdictStatus::ProvenClear,
            survives_unknown: false,
            reason: None,
        }
    }

    pub fn violation(survives_unknown: bool) -> Self {
        Self {
            status: SqlVerdictStatus::Violation,
            survives_unknown,
            reason: None,
        }
    }

    pub fn abstain(reason: &'static str) -> Self {
        Self {
            status: SqlVerdictStatus::Abstain,
            survives_unknown: false,
            reason: Some(reason),
        }
    }
}

enum Branch<'a> {
    MissingText,
    MissingFacts,
    Parsed(Cow<'a, EmbeddedSqlCall>, &'a SqlStatementFileFacts),
}

/// Prepared inputs shared by the owning rule's per-subcheck judgments.
pub struct SqlPolicyVerdictContext<'a> {
    pub facts: &'a CheckFactMap,
    pub embedded: &'a EmbeddedSqlFileFacts,
    pub statements: &'a [SqlStatementFileFacts],
}

/// Attach per-obligation proof to the same prepared call/statement projection
/// the rule already consumes. `checks` contains only enabled subchecks.
pub fn record_sql_policy_verdicts(
    context: SqlPolicyVerdictContext<'_>,
    rule: &str,
    file: &str,
    profile: &EmbeddedSqlOptions,
    checks: &[String],
    advisory: bool,
    judge: impl Fn(&str, &EmbeddedSqlCall, &SqlStatementFileFacts) -> SqlBranchJudgment,
) {
    let SqlPolicyVerdictContext {
        facts,
        embedded,
        statements,
    } = context;
    if !facts.wants_sql_verdicts() || checks.is_empty() {
        return;
    }
    let mut prepared = statements.iter();
    for (index, call) in embedded.calls.iter().enumerate() {
        let branches = call
            .statement_calls()
            .map(|version| match version.sql_text.as_ref() {
                None => Branch::MissingText,
                Some(_) => prepared.next().map_or(Branch::MissingFacts, |statement| {
                    Branch::Parsed(version, statement)
                }),
            })
            .collect::<Vec<_>>();
        let exhaustive = call.kind != EmbeddedSqlKind::Dynamic || !call.variants.is_empty();
        for check in checks {
            let mut statuses = Vec::with_capacity(branches.len());
            let mut residual_reasons = Vec::new();
            let mut positive_survives_unknown = false;
            for branch in &branches {
                let judgment = match branch {
                    Branch::MissingText => SqlBranchJudgment::abstain("unresolved-sql-fragment"),
                    Branch::MissingFacts => {
                        SqlBranchJudgment::abstain("missing-prepared-sql-facts")
                    }
                    Branch::Parsed(version, statement) => {
                        let result = judge(check, version.as_ref(), statement);
                        if statement.parse_failed {
                            residual_reasons.push("invalid-sql".to_string());
                            if call.is_unanalyzable()
                                && result.status == SqlVerdictStatus::Violation
                                && result.survives_unknown
                            {
                                result
                            } else {
                                SqlBranchJudgment::abstain("invalid-sql")
                            }
                        } else {
                            result
                        }
                    }
                };
                positive_survives_unknown |=
                    judgment.status == SqlVerdictStatus::Violation && judgment.survives_unknown;
                if let Some(reason) = judgment.reason {
                    residual_reasons.push(reason.to_string());
                }
                statuses.push(judgment.status);
            }
            if !exhaustive {
                residual_reasons.push("opaque-sql-tail".to_string());
            }
            let status = combine_sql_verdicts(&statuses, exhaustive, positive_survives_unknown);
            residual_reasons.sort();
            residual_reasons.dedup();
            let evidence = match (status, call.variants.is_empty(), exhaustive) {
                (SqlVerdictStatus::Abstain, _, _) => None,
                (_, false, _) => Some(SqlEvidenceKind::ExhaustiveUnion),
                (_, true, false) => Some(SqlEvidenceKind::VerifiedPrefix),
                (_, true, true) => Some(SqlEvidenceKind::Exact),
            };
            facts.record_sql_verdict(SqlObligationVerdict {
                profile: profile.clone(),
                rule: rule.to_string(),
                application: None,
                subcheck: check.clone(),
                file: file.to_string(),
                span: embedded.call_spans[index],
                status,
                advisory,
                evidence,
                known_prefix: call
                    .is_unanalyzable()
                    .then(|| call.sql_text.clone())
                    .flatten(),
                helper_lines: embedded
                    .imported_call_lines
                    .get(&embedded.call_spans[index])
                    .cloned()
                    .unwrap_or_default(),
                residual_reasons,
            });
        }
    }
}
