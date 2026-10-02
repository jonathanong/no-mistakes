mod catalog;
mod check;
mod columns;

use super::{CompiledOptions, RuleFinding};
use crate::codebase::postgres::EmbeddedSqlKind;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Vec<RuleFinding>> {
    if opts.relations.is_empty() && !opts.partition_keys {
        return Ok(Vec::new());
    }
    let prepared =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let catalog = opts
        .schema_catalog_path
        .as_deref()
        .map(|path| prepared.postgres_schema_catalog(path))
        .transpose()?;
    let sql_paths = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)?;
    let mut statements = Vec::new();
    let mut embedded = Vec::new();
    for path in &sql_paths {
        statements.extend(prepared.postgres_statements(path, None)?.iter());
    }
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        embedded.push(prepared.embedded_sql(path, &opts.embedded)?);
        statements.extend(
            prepared
                .postgres_statements(path, Some(&opts.embedded))?
                .iter(),
        );
    }
    let mut findings = Vec::new();
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
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(check::sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for required predicates",
                    Some("unanalyzable"),
                    None,
                ));
            }
        }
    }
    for file in statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(check::sql_finding(
                &rel,
                file.origin_line.max(1),
                "SQL could not be analyzed for required predicates",
                Some("unanalyzable"),
                None,
            ));
            continue;
        }
        findings.extend(check::statement_findings(&rel, file, opts, catalog));
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
