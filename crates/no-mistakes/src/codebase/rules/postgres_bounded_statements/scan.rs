use super::evaluate::offenders;
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::{EmbeddedSqlKind, SqlBoundKind};
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
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
        let source = crate::codebase::rules::read_source(sources, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(finding(
                &rel,
                file.origin_line.max(1),
                "SQL could not be analyzed for bounded statements",
                "unanalyzable",
            ));
            continue;
        }
        let projected = crate::codebase::postgres::project_sql_bounds(file, catalog);
        for bound in projected
            .iter()
            .filter(|bound| opts.statements.contains(&bound.kind))
        {
            for offender in offenders(bound, catalog) {
                // Keep suppression and its audit in the shared layer. A directive
                // on the statement start anchors all of that statement's findings.
                let line = bound
                    .statement_start
                    .and_then(|(line, _)| statement_directive_line(source.as_deref(), line))
                    .or_else(|| statement_directive_line(source.as_deref(), bound.line))
                    .unwrap_or(offender.line)
                    .max(1);
                findings.push(finding(
                    &rel,
                    line,
                    &message(bound.kind, &offender.table),
                    &format!("table:{}", offender.table),
                ));
            }
        }
    }
    // Sorted first: dedup only removes adjacent duplicates, and one relation can be reported
    // by several statements of a file (or arms of one) with others between them.
    crate::codebase::rules::sort_findings(&mut findings);
    findings.dedup();
    let mut findings = opts
        .allow
        .clone()
        .apply(&opts.schema_catalog_path, findings);
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn statement_directive_line(source: Option<&str>, line: usize) -> Option<usize> {
    use crate::codebase::ts_source::{matching_disable_directive, DisableDirective};
    match matching_disable_directive(source?, Some(line.try_into().ok()?), RULE_ID)? {
        DisableDirective::Line { .. } | DisableDirective::NextLine { .. } => Some(line),
        DisableDirective::File { .. } => None,
    }
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
