use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlKind, SchemaCatalog};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_dml: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let catalog = load_catalog(root, opts, sources, prepared)?;
    let mut findings = Vec::new();
    for file in &facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(sql_finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for explicit columns",
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
                "SQL could not be analyzed for explicit columns",
                "unanalyzable",
            ));
            continue;
        }
        for select in &file.selects {
            findings.extend(star_findings(
                &rel,
                &select.star_projections,
                "SELECT *",
                "reads",
                opts,
                &catalog,
            ));
        }
        if opts.check_returning {
            findings.extend(star_findings(
                &rel,
                &file.returning_stars,
                "RETURNING *",
                "returns",
                opts,
                &catalog,
            ));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
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

fn load_catalog(
    root: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    prepared: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> anyhow::Result<Arc<SchemaCatalog>> {
    if let Some(prepared) = prepared {
        let key =
            crate::codebase::postgres::normalize_schema_catalog_path(&opts.schema_catalog_path)?
                .to_string_lossy()
                .into_owned();
        if let Some(entry) = prepared.postgres_schema_catalogs.get(&key) {
            return match entry {
                Ok(catalog) => Ok(Arc::clone(catalog)),
                Err(error) => Err(anyhow::anyhow!(error.to_string())),
            };
        }
    }
    Ok(Arc::new(SchemaCatalog::load(
        root,
        &opts.schema_catalog_path,
        sources,
    )?))
}
