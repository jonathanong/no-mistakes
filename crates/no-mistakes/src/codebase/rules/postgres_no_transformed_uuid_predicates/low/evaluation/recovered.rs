use super::super::cache;
use super::{add_unknown, mark_reason, EvaluationContext, Outcome};
use crate::codebase::postgres::{
    sql_identifier_syntax::{helper_recovery::Recovered, literal_projection::ProjectedLiteral},
    RecoveredSql, SqlVerdictStatus,
};
use crate::codebase::rules::RuleFinding;

pub(super) fn recovered(
    context: &EvaluationContext<'_>,
    proof: &Recovered,
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
) {
    let EvaluationContext {
        application,
        call,
        literals,
        relative,
        options,
        ..
    } = context;
    let mut invalid = false;
    let key = (Some((*application).clone()), call.span);
    if let Some(ProjectedLiteral::Recovered { alternatives }) = literals.get(&key) {
        project_alternatives(context, alternatives, outcome, findings, &mut invalid);
    } else {
        invalid = true;
    }
    if !proof.complete() || invalid {
        outcome.uncertain = true;
        cache::mark_uncertain(&mut outcome.status, &mut outcome.reasons, invalid);
        add_unknown(relative, call.line as usize, options, findings);
    }
}

fn project_alternatives(
    context: &EvaluationContext<'_>,
    alternatives: &[(
        RecoveredSql,
        Result<Vec<crate::codebase::postgres::statements::SqlTransformedColumnUseFact>, ()>,
    )],
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
    invalid: &mut bool,
) {
    let EvaluationContext {
        root,
        caller,
        relative,
        call,
        columns,
        options,
        suppressed,
        ..
    } = context;
    for (sql, facts) in alternatives {
        match facts {
            Ok(uses) => {
                for use_ in uses {
                    match columns.resolve(use_) {
                        Ok((table, true)) => {
                            outcome.status = SqlVerdictStatus::Violation;
                            let mut finding =
                                cache::recovered_finding(root, caller, sql, use_, table);
                            if *suppressed {
                                finding.file = relative.to_string();
                                finding.line = call.line as usize;
                            }
                            findings.push(finding);
                        }
                        Err(reason) => {
                            mark_reason(outcome, reason);
                            add_unknown(relative, call.line as usize, options, findings);
                        }
                        _ => {}
                    }
                }
            }
            Err(()) => {
                *invalid = true;
                outcome.uncertain = true;
            }
        }
    }
}
