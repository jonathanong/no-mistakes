use super::super::{CompiledOptions, cache};
use super::{Outcome, add_unknown, mark_reason};
use crate::codebase::postgres::{
    RecoveredSql, SqlVerdictStatus,
    sql_identifier_syntax::{
        SqlCall,
        helper_recovery::{ApplicationKey, Recovered},
        literal_projection::ProjectedLiteral,
    },
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::rules::postgres_no_transformed_uuid_predicates::evaluate::Columns;
use std::{collections::BTreeMap, path::Path};

type Literals = BTreeMap<(Option<ApplicationKey>, (u32, u32)), super::TransformedLiteral>;

pub(super) fn recovered(
    root: &Path,
    caller: &Path,
    relative: &str,
    application: &ApplicationKey,
    call: &SqlCall,
    proof: &Recovered,
    literals: &Literals,
    columns: &Columns,
    options: &CompiledOptions,
    suppressed: bool,
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
) {
    let mut invalid = false;
    let key = (Some(application.clone()), call.span);
    if let Some(ProjectedLiteral::Recovered { alternatives }) = literals.get(&key) {
        project_alternatives(
            root,
            caller,
            relative,
            call,
            alternatives,
            columns,
            options,
            suppressed,
            outcome,
            findings,
            &mut invalid,
        );
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
    root: &Path,
    caller: &Path,
    relative: &str,
    call: &SqlCall,
    alternatives: &[(
        RecoveredSql,
        Result<Vec<crate::codebase::postgres::statements::SqlTransformedColumnUseFact>, ()>,
    )],
    columns: &Columns,
    options: &CompiledOptions,
    suppressed: bool,
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
    invalid: &mut bool,
) {
    for (sql, facts) in alternatives {
        match facts {
            Ok(uses) => {
                for use_ in uses {
                    match columns.resolve(use_) {
                        Ok((table, true)) => {
                            outcome.status = SqlVerdictStatus::Violation;
                            let mut finding =
                                cache::recovered_finding(root, caller, sql, use_, table);
                            if suppressed {
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
