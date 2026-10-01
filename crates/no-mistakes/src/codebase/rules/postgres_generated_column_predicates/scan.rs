use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::statements::{SqlColumnClause, SqlColumnUseFact, SqlSelectFact};
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlKind};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod catalog;

use catalog::Tracked;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            postgres_dml: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let columns = catalog::column_index(&facts.schema);
    let tracked = catalog::tracked_columns(&facts.schema, opts);
    let mut findings = catalog::stale_extras(&facts.schema, &opts.extras);
    for file in &facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for generated column predicates",
                    "unanalyzable",
                ));
            }
        }
    }
    for file in &facts.statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(sql_finding(
                &rel,
                1,
                "SQL could not be analyzed for generated column predicates",
                "unanalyzable",
            ));
            continue;
        }
        for select in &file.selects {
            findings.extend(select_findings(&rel, select, opts, &tracked, &columns));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn select_findings(
    file: &str,
    select: &SqlSelectFact,
    opts: &CompiledOptions,
    tracked: &[Tracked],
    columns: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for use_ in &select.column_uses {
        if !clause_enabled(use_.clause, opts) {
            continue;
        }
        let Some(table) = owner(use_, select, columns) else {
            continue;
        };
        let Some(column) = tracked
            .iter()
            .find(|item| item.table == table && item.column == use_.column)
        else {
            continue;
        };
        findings.push(sql_finding(
            file,
            use_.line.max(1),
            &message(use_.clause, column),
            &column.column,
        ));
    }
    findings
}

fn owner(
    use_: &SqlColumnUseFact,
    select: &SqlSelectFact,
    columns: &BTreeMap<String, BTreeSet<String>>,
) -> Option<String> {
    if !use_.table.is_empty() {
        return Some(use_.table.clone());
    }
    let mut hits = Vec::new();
    for table in &select.tables {
        let key = table.to_ascii_lowercase();
        let cols = columns.get(&key)?;
        if cols.contains(&use_.column) {
            hits.push(key);
        }
    }
    if hits.len() == 1 {
        hits.pop()
    } else {
        None
    }
}

fn clause_enabled(clause: SqlColumnClause, opts: &CompiledOptions) -> bool {
    match clause {
        SqlColumnClause::Where => opts.where_clause,
        SqlColumnClause::Join => opts.join_clause,
        SqlColumnClause::OrderBy => opts.order_clause,
    }
}

fn message(clause: SqlColumnClause, column: &Tracked) -> String {
    match clause {
        SqlColumnClause::Where => format!(
            "WHERE filters {table}.{name}, which is generated from {function}({source}); compare {source} against a UUIDv7 bound instead so the primary-key index is used",
            table = column.table,
            name = column.column,
            function = column.function,
            source = column.source
        ),
        SqlColumnClause::Join => format!(
            "JOIN ON compares {table}.{name}, which is generated from {function}({source}); compare {source} instead",
            table = column.table,
            name = column.column,
            function = column.function,
            source = column.source
        ),
        SqlColumnClause::OrderBy => format!(
            "ORDER BY {table}.{name} sorts by a column generated from {source}; ORDER BY {source} instead (same order, uses the primary-key index)",
            table = column.table,
            name = column.column,
            source = column.source
        ),
    }
}

fn sql_finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
