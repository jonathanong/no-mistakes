use super::super::declared::{self, Gate};
use super::super::evaluate::{offenders_with_views, variant_offenders_with_views};
use super::super::view_bounds::ViewBounds;
use super::super::{CompiledOptions, RULE_ID};
use super::groups::Groups;
use super::messages::finding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SchemaCatalog, SqlBoundFact, SqlStatementFileFacts};
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};
use std::collections::HashSet;

pub(super) struct FileInput<'a, 'src> {
    pub(super) prepared: &'a dyn SqlFactContext,
    pub(super) opts: &'a CompiledOptions,
    pub(super) catalog: &'a SchemaCatalog,
    pub(super) views: &'a ViewBounds,
    pub(super) findings: &'a mut PostgresFindings,
    pub(super) advice: &'a mut PostgresFindings,
    pub(super) groups: &'a mut Groups,
    pub(super) advice_groups: &'a mut Groups,
    pub(super) attribution: &'a mut VariantFindingDedup<'src>,
    pub(super) native_statement_count: usize,
}

pub(super) fn file(
    input: FileInput<'_, '_>,
    start_index: usize,
    files: &[&SqlStatementFileFacts],
    rel: &str,
    source: Option<&str>,
) {
    let FileInput {
        prepared,
        opts,
        catalog,
        views,
        findings,
        advice,
        groups,
        advice_groups,
        attribution,
        native_statement_count,
    } = input;
    let mut anchors = Vec::new();
    for facts in files {
        let projected = crate::codebase::postgres::project_sql_bounds(facts, catalog);
        anchors.extend(
            projected
                .iter()
                .filter(|bound| checked(opts, bound))
                .map(anchor_line),
        );
    }
    let plan = declared::plan(source, &anchors);
    let mut offender_anchors = HashSet::new();
    if !plan.is_empty() {
        for facts in files {
            let projected = crate::codebase::postgres::project_sql_bounds(facts, catalog);
            for bound in projected.iter().filter(|bound| checked(opts, bound)) {
                let anchor = anchor_line(bound);
                if matches!(plan.gate(anchor), Some(Gate::Valid { .. }))
                    && !offenders(facts, bound, catalog, views).is_empty()
                {
                    offender_anchors.insert(anchor);
                }
            }
        }
        for (line, text) in plan.unmatched() {
            findings.push(diagnostic(rel, *line, text));
        }
    }
    let mut settled = HashSet::new();
    for (offset, facts) in files.iter().enumerate() {
        let projected = crate::codebase::postgres::project_sql_bounds(facts, catalog);
        for (bound_index, bound) in projected
            .iter()
            .enumerate()
            .filter(|(_, bound)| checked(opts, bound))
        {
            let anchor = anchor_line(bound);
            if let Some(gate) = plan.gate(anchor) {
                if settled.insert(anchor) {
                    apply_gate(
                        prepared,
                        findings,
                        rel,
                        gate,
                        offender_anchors.contains(&anchor),
                    );
                }
                if matches!(gate, Gate::Valid { .. }) {
                    continue;
                }
            }
            let advise = bound.body.is_some()
                && opts.materialized_bodies == super::super::bodies::Policy::Advise;
            let (output, selected_groups) = if advise {
                (&mut *advice, &mut *advice_groups)
            } else {
                (&mut *findings, &mut *groups)
            };
            super::emit::offenders(super::emit::Offenders {
                output,
                groups: selected_groups,
                attribution,
                facts,
                source,
                rel,
                bound_index,
                bound,
                file_index: start_index + offset,
                native_statement_count,
                offenders: offenders(facts, bound, catalog, views),
            });
        }
    }
}

fn apply_gate(
    prepared: &dyn SqlFactContext,
    findings: &mut PostgresFindings,
    rel: &str,
    gate: &Gate,
    offenders_present: bool,
) {
    match gate {
        Gate::Valid {
            line,
            rows,
            justification,
        } => {
            if offenders_present {
                prepared.record_accepted_statement_bound(RULE_ID, rel, *line, *rows, justification);
            } else {
                findings.push(diagnostic(rel, *line, &declared::stale_message(*rows)));
            }
        }
        Gate::Invalid { line, message } => findings.push(diagnostic(rel, *line, message)),
        Gate::Blocked { findings: blocked } => {
            for (line, message) in blocked {
                findings.push(diagnostic(rel, *line, message));
            }
        }
    }
}

fn offenders(
    facts: &SqlStatementFileFacts,
    bound: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<super::super::evaluate::Offender> {
    if facts.variant_locations.is_some() {
        variant_offenders_with_views(bound, catalog, views)
    } else {
        offenders_with_views(bound, catalog, views)
    }
}

fn checked(opts: &CompiledOptions, bound: &SqlBoundFact) -> bool {
    opts.statements.contains(&bound.kind)
        && !(bound.body.is_some()
            && opts.materialized_bodies == super::super::bodies::Policy::Ignore)
}

fn anchor_line(bound: &SqlBoundFact) -> usize {
    bound
        .statement_start
        .map(|(line, _)| line)
        .unwrap_or(bound.line)
        .max(1)
}

fn diagnostic(rel: &str, line: usize, message: &str) -> super::super::RuleFinding {
    finding(rel, line, message, "declared-bound")
}
