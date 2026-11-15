use super::super::declared::{self, Gate};
use super::super::evaluate::{
    judgment_with_views, variant_judgment_with_views, Judgment,
};
use super::super::view_bounds::ViewBounds;
use super::super::CompiledOptions;
use super::groups::Groups;
use super::messages::finding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{SchemaCatalog, SqlBoundFact, SqlStatementFileFacts};
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};
use std::collections::HashSet;

#[path = "judge_declared.rs"]
mod declared_gate;

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
    let anchors = statement_anchors(opts, catalog, files);
    let plan = declared::plan(source, &anchors);
    let mut offender_anchors = HashSet::new();
    if !plan.is_empty() {
        for facts in files {
            let projected = crate::codebase::postgres::project_sql_bounds(facts, catalog);
            for bound in projected.iter().filter(|bound| checked(opts, bound)) {
                let anchor = anchor_line(bound);
                if matches!(plan.gate(anchor), Some(Gate::Valid { .. }))
                    && !judgment(facts, bound, catalog, views).offenders.is_empty()
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
                    declared_gate::apply(
                        prepared,
                        findings,
                        source,
                        rel,
                        anchor,
                        gate,
                        offender_anchors.contains(&anchor),
                    );
                }
                if declared_gate::clears(source, anchor, gate) {
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
            let judged = judgment(facts, bound, catalog, views);
            super::domain_note::record(prepared, opts, rel, source, &judged.accepts);
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
                offenders: judged.offenders,
                domain_rows: views.domain_rows,
            });
        }
    }
}

fn statement_anchors(
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
    files: &[&SqlStatementFileFacts],
) -> Vec<usize> {
    let mut anchors = Vec::new();
    let mut seen = HashSet::new();
    for (index, facts) in files.iter().enumerate() {
        let projected = crate::codebase::postgres::project_sql_bounds(facts, catalog);
        for bound in projected.iter().filter(|bound| checked(opts, bound)) {
            let identity = (
                index,
                bound.statement_start.unwrap_or((bound.line, bound.column)),
            );
            if seen.insert(identity) {
                anchors.push(anchor_line(bound));
            }
        }
    }
    anchors
}

fn judgment(
    facts: &SqlStatementFileFacts,
    bound: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Judgment {
    if facts.variant_locations.is_some() {
        variant_judgment_with_views(bound, catalog, views)
    } else {
        judgment_with_views(bound, catalog, views)
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

pub(super) fn diagnostic(rel: &str, line: usize, message: &str) -> super::super::RuleFinding {
    finding(rel, line, message, "declared-bound")
}
