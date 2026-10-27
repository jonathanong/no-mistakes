use crate::codebase::check_facts::SqlFactContext;
mod catalog;
mod check;
mod columns;
mod verdicts;

use super::CompiledOptions;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&dyn SqlFactContext>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    if opts.relations.is_empty() && !opts.partition_keys {
        return Ok(crate::codebase::rules::PostgresFindings::default());
    }
    let context =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let prepared = context.prepared();
    let catalog = opts
        .schema_catalog_path
        .as_deref()
        .map(|path| prepared.postgres_schema_catalog(path))
        .transpose()?;
    let sql_paths = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)?;
    let mut statements = Vec::new();
    let mut embedded = Vec::new();
    for path in &sql_paths {
        statements.extend(context.postgres_statements(path, None)?.iter());
    }
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        embedded.push(context.embedded_sql(path, &opts.embedded)?);
        statements.extend(
            context
                .postgres_statements(path, Some(&opts.embedded))?
                .iter(),
        );
    }
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    if context.wants_sql_verdicts() {
        verdicts::record(root, opts, &embedded, catalog, context)?;
    }
    if let Some(catalog) = &catalog {
        if opts.partition_keys {
            if let Some(path) = &opts.schema_catalog_path {
                findings.extend(catalog::catalog_findings(path, catalog, opts));
            }
        }
    }
    for file in embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(check::sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    &context.sql_unanalyzable_message(
                        "executed SQL is not statically recoverable for required predicates",
                    ),
                    Some("unanalyzable"),
                    None,
                ));
            }
        }
    }
    for file in statements {
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            for (site, line) in file.parse_failure_sites() {
                dedup.push(
                    &mut findings,
                    file,
                    site,
                    source.as_deref(),
                    check::sql_finding(
                        &rel,
                        line,
                        "SQL could not be analyzed for required predicates",
                        Some("unanalyzable"),
                        None,
                    ),
                );
            }
        }
        for (site, finding) in check::statement_findings(&rel, file, opts, catalog) {
            dedup.push_merged(&mut findings, file, site, source.as_deref(), finding);
        }
    }
    findings.sort();
    Ok(findings)
}
