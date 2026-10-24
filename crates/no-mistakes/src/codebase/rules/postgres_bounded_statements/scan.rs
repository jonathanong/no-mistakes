use super::evaluate::{offenders_with_views, variant_offenders_with_views};
use super::view_bounds::ViewBounds;
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::SqlBoundKind;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};
mod verdicts;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&dyn SqlFactContext>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let inputs = crate::codebase::rules::collect_prepared_sql_inputs(
        root,
        files,
        &opts.schema,
        &opts.schema_catalog_path,
        &opts.embedded,
        prepared.map(|facts| facts.prepared()),
    )?;
    let catalog = inputs.catalog;
    let views = ViewBounds::compile(&opts.view_bounds, catalog)?;
    let statements = inputs.statements;
    let embedded = inputs.embedded;
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if let Some(prepared) = prepared.filter(|facts| facts.wants_sql_verdicts()) {
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
    for file in statements {
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            dedup.push(
                &mut findings,
                file,
                crate::codebase::postgres::statements::SqlFactSite::Origin,
                source.as_deref(),
                finding(
                    &rel,
                    file.origin_line.max(1),
                    "SQL could not be analyzed for bounded statements",
                    "unanalyzable",
                ),
            );
            continue;
        }
        let projected = crate::codebase::postgres::project_sql_bounds(file, catalog);
        for (bound_index, bound) in projected
            .iter()
            .enumerate()
            .filter(|(_, bound)| opts.statements.contains(&bound.kind))
        {
            let evaluate = if file.variant_locations.is_some() {
                variant_offenders_with_views
            } else {
                offenders_with_views
            };
            for offender in evaluate(bound, catalog, &views) {
                // Keep suppression and its audit in the shared layer. A directive
                // on the statement start anchors all of that statement's findings.
                let line = bound
                    .statement_start
                    .and_then(|(line, _)| statement_directive_line(source.as_deref(), line))
                    .or_else(|| statement_directive_line(source.as_deref(), bound.line))
                    .unwrap_or(offender.line)
                    .max(1);
                let site = file
                    .variant_locations
                    .as_ref()
                    .and_then(|locations| {
                        locations
                            .bound_table_sites(bound_index, &offender.table, offender.line)
                            .find(|(_, position)| position.sql_column == offender.column)
                            .map(|(site, _)| site)
                    })
                    .unwrap_or(crate::codebase::postgres::statements::SqlFactSite::Bound(
                        bound_index,
                    ));
                dedup.push(
                    &mut findings,
                    file,
                    site,
                    source.as_deref(),
                    finding(
                        &rel,
                        line,
                        &message(bound.kind, &offender.table, &offender.object),
                        &offender.object.to_string(),
                    ),
                );
            }
        }
    }
    // Sorted first: dedup only removes adjacent duplicates, and one relation can be reported
    // by several statements of a file (or arms of one) with others between them.
    findings.sort();
    let mut findings = opts
        .allow
        .clone()
        .apply_located(&opts.schema_catalog_path, findings);
    findings.sort();
    Ok(findings)
}

pub(super) fn statement_directive_line(source: Option<&str>, line: usize) -> Option<usize> {
    use crate::codebase::ts_source::{matching_disable_directive, DisableDirective};
    match matching_disable_directive(source?, Some(line.try_into().ok()?), RULE_ID)? {
        DisableDirective::Line { .. } | DisableDirective::NextLine { .. } => Some(line),
        DisableDirective::File { .. } => None,
    }
}

fn message(
    kind: SqlBoundKind,
    table: &str,
    object: &crate::codebase::postgres::CatalogObjectRef,
) -> String {
    if matches!(object, crate::codebase::postgres::CatalogObjectRef::View(_)) {
        return format!("{kind:?} can read unbounded rows from view {table}; add a row cap or configure viewBounds as bounded only when its result is finite");
    }
    match kind {
        SqlBoundKind::Select => format!(
            "SELECT can return every row of {table}; add a LIMIT, look rows up by a primary or unique key, or allow the table with a reason"
        ),
        SqlBoundKind::Update => format!(
            "UPDATE can change every row of {table}; pick the target rows in a CTE or subquery with a LIMIT, or match a primary or unique key"
        ),
        SqlBoundKind::Delete => format!(
            "DELETE can remove every row of {table}; pick the target rows in a CTE or subquery with a LIMIT, or match a primary or unique key"
        ),
    }
}

fn finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
