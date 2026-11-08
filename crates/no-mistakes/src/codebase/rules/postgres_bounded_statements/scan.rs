use super::evaluate::{offenders_with_views, variant_offenders_with_views};
use super::view_bounds::ViewBounds;
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::SqlBoundKind;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};
mod groups;
mod messages;
mod verdicts;
use messages::{finding, message};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&dyn SqlFactContext>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let prepared =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let inputs = crate::codebase::rules::collect_prepared_sql_inputs(
        root,
        files,
        &opts.schema,
        &opts.schema_catalog_path,
        &opts.embedded,
        Some(prepared),
    )?;
    let catalog = inputs.catalog;
    let mut views = ViewBounds::compile(&opts.view_bounds, catalog)?;
    views.keys = super::column_bounds::ColumnBounds::compile(&opts.column_bounds, catalog)?;
    let native_statement_count = inputs.native_statement_count;
    let statements = inputs.statements;
    let embedded = inputs.embedded;
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    let mut attribution = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if prepared.wants_sql_verdicts() {
        verdicts::record(root, opts, &embedded, catalog, &views, prepared)?;
    }
    for file in embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for bounded statements",
                    "unanalyzable",
                ));
            }
        }
    }
    let mut groups = groups::Groups::default();
    let mut advice_groups = groups::Groups::default();
    let mut advice = crate::codebase::rules::PostgresFindings::default();
    for (file_index, file) in statements.into_iter().enumerate() {
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path);
        if opts.advise_unanalyzable && file_index < native_statement_count {
            crate::codebase::rules::native_sql_advice::record_parse_failures(
                prepared,
                RULE_ID,
                &opts.embedded,
                &rel,
                source.as_deref().unwrap_or_default(),
                file,
            );
        }
        if opts.fail_unanalyzable && file.parse_failed {
            for (site, line) in file.parse_failure_sites() {
                dedup.push(
                    &mut findings,
                    file,
                    site,
                    source.as_deref(),
                    finding(
                        &rel,
                        line,
                        "SQL could not be analyzed for bounded statements",
                        "unanalyzable",
                    ),
                );
            }
        }
        let projected = crate::codebase::postgres::project_sql_bounds(file, catalog);
        for (bound_index, bound) in projected
            .iter()
            .enumerate()
            .filter(|(_, bound)| opts.statements.contains(&bound.kind))
        {
            if bound.body.is_some() && opts.materialized_bodies == super::bodies::Policy::Ignore {
                continue;
            }
            let evaluate = if file.variant_locations.is_some() {
                variant_offenders_with_views
            } else {
                offenders_with_views
            };
            let (output, selected_groups) = if bound.body.is_some()
                && opts.materialized_bodies == super::bodies::Policy::Advise
            {
                (&mut advice, &mut advice_groups)
            } else {
                (&mut findings, &mut groups)
            };
            let group_start = output.len();
            for offender in evaluate(bound, catalog, &views) {
                // Keep suppression and its audit in the shared layer. A directive
                // on the statement start anchors all of that statement's findings.
                let line = bound
                    .statement_start
                    .and_then(|(line, _)| statement_directive_line(source.as_deref(), line))
                    .or_else(|| statement_directive_line(source.as_deref(), bound.line))
                    .unwrap_or(if bound.body.is_some() {
                        bound.line
                    } else {
                        offender.line
                    })
                    .max(1);
                let body_message = bound
                    .body
                    .as_ref()
                    .map(|body| super::bodies::message(body, &offender.table));
                let site = file
                    .variant_locations
                    .as_ref()
                    .filter(|_| bound.body.is_none())
                    .and_then(|locations| {
                        locations
                            .bound_table_sites(bound_index, &offender.table, offender.line)
                            .find(|(_, position)| position.sql_column == offender.column)
                            .map(|(site, _)| site)
                    })
                    .unwrap_or(crate::codebase::postgres::statements::SqlFactSite::Bound(
                        bound_index,
                    ));
                attribution.push_attributed(
                    output,
                    file,
                    site,
                    source.as_deref(),
                    finding(
                        &rel,
                        line,
                        &body_message.unwrap_or_else(|| {
                            message(bound.kind, &offender.table, &offender.object)
                        }),
                        &offender.object.to_string(),
                    ),
                );
            }
            selected_groups.collect(
                output,
                group_start,
                file,
                bound_index,
                bound,
                (file_index >= native_statement_count).then_some(file_index),
            );
        }
    }
    groups.finish(&mut findings, root, sources, &opts.allow);
    advice_groups.finish(&mut advice, root, sources, &opts.allow);
    Ok(super::bodies::finish(
        prepared, opts, root, sources, findings, advice,
    ))
}

pub(super) fn statement_directive_line(source: Option<&str>, line: usize) -> Option<usize> {
    use crate::codebase::ts_source::{matching_disable_directive, DisableDirective};
    match matching_disable_directive(source?, Some(line.try_into().ok()?), RULE_ID)? {
        DisableDirective::Line { .. } | DisableDirective::NextLine { .. } => Some(line),
        DisableDirective::File { .. } => None,
    }
}
