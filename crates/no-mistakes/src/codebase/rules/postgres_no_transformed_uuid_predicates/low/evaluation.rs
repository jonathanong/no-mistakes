use super::{cache, CompiledOptions};
mod recovered;
use crate::codebase::postgres::{
    sql_identifier_syntax::{
        helper_recovery::ApplicationKey, literal_projection::ProjectedLiteral, Input, SqlCall,
        TransformedLiteral,
    },
    SqlVerdictStatus,
};
use crate::codebase::rules::postgres_no_transformed_uuid_predicates::evaluate::Columns;
use crate::codebase::rules::RuleFinding;
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

pub(super) struct EvaluationContext<'a> {
    pub root: &'a Path,
    pub caller: &'a Path,
    pub relative: &'a str,
    pub application: &'a ApplicationKey,
    pub call: &'a SqlCall,
    pub literals: &'a Literals,
    pub columns: &'a Columns<'a>,
    pub options: &'a CompiledOptions,
    pub suppressed: bool,
}

pub(super) fn evaluate(context: &EvaluationContext<'_>) -> (Outcome, Vec<RuleFinding>) {
    let EvaluationContext {
        relative,
        call,
        options,
        ..
    } = context;
    let mut outcome = Outcome::default();
    let mut findings = Vec::new();
    match &call.input {
        Input::Sql { source_span, .. } => {
            direct(context, *source_span, &mut outcome, &mut findings)
        }
        Input::Recovered(proof) => {
            recovered::recovered(context, proof, &mut outcome, &mut findings)
        }
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
    context: &EvaluationContext<'_>,
    source_span: (u32, u32),
    outcome: &mut Outcome,
    findings: &mut Vec<RuleFinding>,
) {
    let EvaluationContext {
        relative,
        literals,
        columns,
        options,
        suppressed,
        call,
        ..
    } = context;
    let call_line = call.line as usize;
    match literals.get(&(None, source_span)) {
        Some(ProjectedLiteral::Direct(Ok(Some(uses)))) => {
            for use_ in uses {
                match columns.resolve(use_) {
                    Ok((table, true)) => {
                        outcome.status = SqlVerdictStatus::Violation;
                        let mut finding = super::super::scan::finding(relative, table, use_);
                        if *suppressed {
                            finding.line = call_line;
                        }
                        findings.push(finding);
                    }
                    Err(reason) => {
                        mark_reason(outcome, reason);
                        if options.fail_unanalyzable {
                            let line = if *suppressed { call_line } else { use_.line };
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
