use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::statements::{SqlColumnClause, SqlColumnUseFact, SqlFactSite};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod catalog;
use catalog::Tracked;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    query_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let sql_paths = crate::codebase::postgres::postgres_sql_paths(root, files, &opts.schema)?;
    let schema = sql_paths
        .iter()
        .map(|path| prepared.postgres_schema_file(path))
        .collect::<anyhow::Result<Vec<_>>>()?;
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
    let queries = crate::codebase::check_facts::PathMembership::new(query_files);
    let live = catalog::live_columns(&schema);
    let columns = catalog::column_index(&live);
    let tracked = catalog::tracked_columns(&live, opts);
    let mut findings = catalog::stale_extras(&live, &opts.extras);
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    if tracked.is_empty() {
        return Ok(findings);
    }
    for file in embedded.iter().filter(|file| queries.contains(&file.path)) {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for generated column predicates",
                    "unanalyzable",
                ));
            }
        }
    }
    for file in statements
        .iter()
        .filter(|file| queries.contains(&file.path))
    {
        let rel = relative_slash_path(root, &file.path);
        let source = crate::codebase::rules::read_source(sources, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            dedup.push(
                &mut findings,
                file,
                SqlFactSite::Origin,
                source.as_deref(),
                sql_finding(
                    &rel,
                    file.origin_line.max(1),
                    "SQL could not be analyzed for generated column predicates",
                    "unanalyzable",
                ),
            );
            continue;
        }
        for (select_index, select) in file.selects.iter().enumerate() {
            for (column_index, use_) in select.column_uses.iter().enumerate() {
                for finding in
                    column_findings(&rel, std::slice::from_ref(use_), opts, &tracked, &columns)
                {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::Column(select_index, column_index),
                        source.as_deref(),
                        finding,
                    );
                }
            }
        }
        for (index, use_) in file.mutation_column_uses.iter().enumerate() {
            for finding in
                column_findings(&rel, std::slice::from_ref(use_), opts, &tracked, &columns)
            {
                dedup.push(
                    &mut findings,
                    file,
                    SqlFactSite::MutationColumn(index),
                    source.as_deref(),
                    finding,
                );
            }
        }
    }
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

fn column_findings(
    file: &str,
    uses: &[SqlColumnUseFact],
    opts: &CompiledOptions,
    tracked: &[Tracked],
    columns: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for use_ in uses {
        if !clause_enabled(use_.clause, opts) {
            continue;
        }
        let Some(table) = owner(use_, columns) else {
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

fn owner(use_: &SqlColumnUseFact, columns: &BTreeMap<String, BTreeSet<String>>) -> Option<String> {
    if !use_.table.is_empty() {
        return Some(use_.table.clone());
    }
    let mut hits = Vec::new();
    for table in use_.candidate_tables.as_ref()? {
        let key = table.clone();
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
    let context = match clause {
        SqlColumnClause::Where => format!("WHERE filters {}.{}", column.table, column.column),
        SqlColumnClause::Join => format!("JOIN ON compares {}.{}", column.table, column.column),
        SqlColumnClause::OrderBy => format!(
            "ORDER BY {}.{} sorts by a column generated from {}",
            column.table, column.column, column.source
        ),
    };
    let expression = column
        .function
        .as_ref()
        .map(|function| format!("{function}({})", column.source))
        .unwrap_or_else(|| format!("a configured expression using {}", column.source));
    format!("{context}, which is generated from {expression}; consider using {source} directly with a bound or ordering that preserves the generation expression's semantics and uses an appropriate index", source = column.source)
}

fn sql_finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        source_offset: None,
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
