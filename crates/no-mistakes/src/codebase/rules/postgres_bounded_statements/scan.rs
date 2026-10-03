use super::evaluate::offenders;
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::{EmbeddedSqlKind, SqlBoundKind};
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Vec<RuleFinding>> {
    let prepared =
        prepared.ok_or_else(|| anyhow::anyhow!("prepared PostgreSQL facts are required"))?;
    let catalog = prepared.postgres_schema_catalog(&opts.schema_catalog_path)?;
    let paths = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)?;
    let mut statements = Vec::new();
    let mut embedded = Vec::new();
    for path in &paths {
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
    for file in embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
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
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(finding(
                &rel,
                file.origin_line.max(1),
                "SQL could not be analyzed for bounded statements",
                "unanalyzable",
            ));
            continue;
        }
        for bound in file
            .bounds
            .iter()
            .filter(|bound| opts.statements.contains(&bound.kind))
        {
            for offender in offenders(bound, catalog) {
                findings.push(finding(
                    &rel,
                    offender.line.max(1),
                    &message(bound.kind, &offender.table),
                    &format!("table:{}", offender.table),
                ));
            }
        }
    }
    findings.dedup();
    let mut findings = opts
        .allow
        .clone()
        .apply(&opts.schema_catalog_path, findings);
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn message(kind: SqlBoundKind, table: &str) -> String {
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
