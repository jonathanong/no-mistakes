use super::{CompiledOptions, cache};
mod recovered;
use crate::codebase::postgres::{
    SqlVerdictStatus,
    sql_identifier_syntax::{
        Input, SqlCall, TransformedLiteral, helper_recovery::ApplicationKey,
        literal_projection::ProjectedLiteral,
    },
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::rules::postgres_no_transformed_uuid_predicates::evaluate::Columns;
use std::{collections::BTreeMap, path::Path};

type Literals = BTreeMap<(Option<ApplicationKey>, (u32, u32)), TransformedLiteral>;

pub(super) struct Outcome {
    pub status: SqlVerdictStatus,
    pub uncertain: bool,
    pub reasons: Vec<String>,
}
impl Default for Outcome {
    fn default() -> Self {
        Self {
            status: SqlVerdictStatus::ProvenClear,
            uncertain: false,
            reasons: Vec::new(),
        }
    }
}

pub(super) fn evaluate(
    root: &Path,
    caller: &Path,
    relative: &str,
    application: &ApplicationKey,
    call: &SqlCall,
    literals: &Literals,
    columns: &Columns,
    options: &CompiledOptions,
    suppressed: bool,
) -> (Outcome, Vec<RuleFinding>) {
    let mut outcome = Outcome::default();
    let mut findings = Vec::new();
    match &call.input {
        Input::Sql { source_span, .. } => direct(
            relative,
            call.line as usize,
            *source_span,
            literals,
            columns,
            options,
            suppressed,
            &mut outcome,
            &mut findings,
        ),
        Input::Recovered(proof) => recovered::recovered(
            root,
            caller,
            relative,
            application,
            call,
            proof,
            literals,
            columns,
            options,
            suppressed,
            &mut outcome,
            &mut findings,
        ),
        _ => {
            cache::mark_input_unknown(
                &mut outcome.status,
                &mut outcome.uncertain,
                &mut outcome.reasons,
            );
            add_unknown(relative, call.line as usize, options, &mut findings);
        }
    }
    (outcome, findings)
}

fn direct(
    relative: &str,
    call_line: usize,
    source_span: (u32, u32),
    literals: &Literals,
    columns: &Columns,
    options: &CompiledOptions,
    suppressed: bool,
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
) {
    match literals.get(&(None, source_span)) {
        Some(ProjectedLiteral::Direct(Ok(Some(uses)))) => {
            for use_ in uses {
                match columns.resolve(use_) {
                    Ok((table, true)) => {
                        outcome.status = SqlVerdictStatus::Violation;
                        let mut finding = super::super::scan::finding(relative, table, use_);
                        if suppressed {
                            finding.line = call_line;
                        }
                        findings.push(finding);
                    }
                    Err(reason) => {
                        mark_reason(outcome, reason);
                        if options.fail_unanalyzable {
                            let line = if suppressed { call_line } else { use_.line };
                            findings.push(super::super::scan::unknown(relative, line));
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(ProjectedLiteral::Direct(Err(())))
        | Some(ProjectedLiteral::Direct(Ok(None)))
        | None => {
            mark_reason(
                outcome,
                "SQL is unanalyzable at effort low; pass a literal string or interpolation-free template directly to the imported executor or through one local const, or use effort max",
            );
            add_unknown(relative, call_line, options, findings);
        }
        _ => {
            cache::mark_input_unknown(
                &mut outcome.status,
                &mut outcome.uncertain,
                &mut outcome.reasons,
            );
            add_unknown(relative, call_line, options, findings);
        }
    }
}

fn mark_reason(outcome: &mut Outcome, reason: impl Into<String>) {
    if outcome.status != SqlVerdictStatus::Violation {
        outcome.status = SqlVerdictStatus::Abstain;
    }
    outcome.uncertain = true;
    outcome.reasons.push(reason.into());
}

fn add_unknown(
    relative: &str,
    line: usize,
    options: &CompiledOptions,
    findings: &mut Vec<RuleFinding>,
) {
    if options.fail_unanalyzable {
        findings.push(super::super::scan::unknown(relative, line));
    }
}
