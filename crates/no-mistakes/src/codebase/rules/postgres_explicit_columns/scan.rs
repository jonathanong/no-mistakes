use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::SchemaCatalog;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
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
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    for file in embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for explicit columns",
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
                SqlFactSite::Origin,
                source.as_deref(),
                sql_finding(
                    &rel,
                    file.origin_line.max(1),
                    "SQL could not be analyzed for explicit columns",
                    "unanalyzable",
                ),
            );
            continue;
        }
        for (select_index, select) in file.selects.iter().enumerate() {
            for (star_index, star) in select.star_projections.iter().enumerate() {
                for finding in star_findings(
                    &rel,
                    std::slice::from_ref(star),
                    "SELECT *",
                    "reads",
                    opts,
                    catalog,
                ) {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::Star(select_index, star_index),
                        source.as_deref(),
                        finding,
                    );
                }
            }
        }
        if opts.check_returning {
            for (star_index, star) in file.returning_stars.iter().enumerate() {
                for finding in star_findings(
                    &rel,
                    std::slice::from_ref(star),
                    "RETURNING *",
                    "returns",
                    opts,
                    catalog,
                ) {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::ReturningStar(star_index),
                        source.as_deref(),
                        finding,
                    );
                }
            }
        }
    }
    findings.sort();
    Ok(findings)
}

fn star_findings(
    file: &str,
    stars: &[SqlStarProjectionFact],
    kind: &str,
    verb: &str,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for star in stars {
        if let Some(function) = &star.within_function {
            if opts
                .whole_row_functions
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(function))
            {
                continue;
            }
        }
        let Some(text) = message(kind, verb, star, opts, catalog) else {
            continue;
        };
        findings.push(sql_finding(file, star.line.max(1), &text, &star.relation));
    }
    findings
}

fn message(
    kind: &str,
    verb: &str,
    star: &SqlStarProjectionFact,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> Option<String> {
    let listed = opts
        .relations
        .iter()
        .any(|relation| relation.eq_ignore_ascii_case(&star.relation));
    if listed {
        return Some(format!(
            "{kind} {verb} all columns of {table}, which is configured in relations; list the columns the caller uses",
            table = star.relation
        ));
    }
    let info = catalog.relation(&star.relation)?;
    if opts.max_columns == 0 {
        return Some(format!(
            "{kind} {verb} every column of {table}, including columns added later; list the columns the caller uses",
            table = star.relation
        ));
    }
    let width = info.columns.len();
    if width <= opts.max_columns as usize {
        return None;
    }
    Some(format!(
        "{kind} {verb} all {width} columns of {table} (limit {limit}); list the columns the caller uses",
        table = star.relation,
        limit = opts.max_columns
    ))
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
