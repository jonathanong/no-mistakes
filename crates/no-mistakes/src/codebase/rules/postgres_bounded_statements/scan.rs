use super::view_bounds::ViewBounds;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};
mod emit;
mod groups;
mod judge;
mod messages;
mod verdicts;
use messages::finding;

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
    let mut start = 0;
    while start < statements.len() {
        let mut end = start + 1;
        while end < statements.len() && statements[end].path == statements[start].path {
            end += 1;
        }
        let rel = relative_slash_path(root, &statements[start].path);
        let source = crate::codebase::rules::read_language_source(sources, &statements[start].path);
        for (offset, file) in statements[start..end].iter().copied().enumerate() {
            let file_index = start + offset;
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
        }
        judge::file(
            judge::FileInput {
                prepared,
                opts,
                catalog,
                views: &views,
                findings: &mut findings,
                advice: &mut advice,
                groups: &mut groups,
                advice_groups: &mut advice_groups,
                attribution: &mut attribution,
                native_statement_count,
            },
            start,
            &statements[start..end],
            &rel,
            source.as_deref(),
        );
        start = end;
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
