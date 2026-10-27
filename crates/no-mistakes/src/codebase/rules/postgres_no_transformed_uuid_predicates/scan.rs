mod raw_advice;
use super::evaluate::Columns;
use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::statements::{
    SqlFactSite, SqlTransformedColumnClause as Clause, SqlTransformedColumnUseFact,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    context: &dyn SqlFactContext,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let inputs = crate::codebase::rules::collect_prepared_sql_inputs(
        root,
        files,
        &opts.schema,
        &opts.schema_catalog_path,
        &opts.embedded,
        Some(context),
    )?;
    let columns = Columns::new(inputs.catalog);
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    for file in inputs.embedded {
        let rel = relative_slash_path(root, &file.path);
        if context.wants_sql_verdicts() || opts.advise_unanalyzable {
            crate::codebase::postgres::record_sql_policy_verdicts(
                crate::codebase::postgres::SqlPolicyVerdictContext {
                    facts: context,
                    embedded: file,
                    statements: context
                        .prepared()
                        .postgres_statements(&file.path, Some(&opts.embedded))?,
                },
                RULE_ID,
                &rel,
                &opts.embedded,
                &["uuid-predicates".into()],
                opts.advise_unanalyzable,
                |_, _, statement| columns.judge(&statement.transformed_column_uses),
            );
        }
        for call in &file.calls {
            if opts.fail_unanalyzable && call.is_unanalyzable() {
                findings.push(unknown(&rel, call.line.max(1) as usize));
            }
        }
    }
    for statement in inputs.statements {
        let rel = relative_slash_path(root, &statement.path);
        let source = crate::codebase::rules::read_source(sources, &statement.path);
        let mut residuals = std::collections::BTreeMap::new();
        for (site, line) in statement.parse_failure_sites() {
            residuals.insert(line, "invalid-sql");
            if opts.fail_unanalyzable {
                dedup.push(
                    &mut findings,
                    statement,
                    site,
                    source.as_deref(),
                    unknown(&rel, line),
                );
            }
        }
        for (index, use_) in statement.transformed_column_uses.iter().enumerate() {
            match columns.resolve(use_) {
                Ok((table, true)) => dedup.push(
                    &mut findings,
                    statement,
                    SqlFactSite::TransformedColumn(index),
                    source.as_deref(),
                    finding(&rel, table, use_),
                ),
                Err(reason) => {
                    residuals.entry(use_.line).or_insert(reason);
                    if opts.fail_unanalyzable {
                        dedup.push(
                            &mut findings,
                            statement,
                            SqlFactSite::TransformedColumn(index),
                            source.as_deref(),
                            unknown(&rel, use_.line),
                        );
                    }
                }
                _ => {}
            }
        }
        if opts.advise_unanalyzable && rel.ends_with(".sql") {
            let advice = raw_advice::RawSqlAdvice::new(source.as_deref().unwrap_or_default());
            for (line, reason) in residuals {
                advice.record(context, opts, &rel, line, reason);
            }
        }
    }
    findings.sort();
    Ok(findings)
}
pub(super) fn finding(file: &str, table: &str, use_: &SqlTransformedColumnUseFact) -> RuleFinding {
    let clause = match use_.clause {
        Clause::Where => "WHERE",
        Clause::Join => "JOIN ON",
        Clause::Having => "HAVING",
    };
    let line = use_.line.max(1);
    RuleFinding {
        rule: RULE_ID.into(), file: file.into(), line,
        message: format!("{file}:{line}: {clause} applies {} to UUID column {table}.{}; keep the UUID column bare so an ordinary column index can support the predicate. Cast the parameter to uuid, or compare the UUID column to a precomputed bound that preserves the intended semantics", use_.transformation, use_.column),
        import: None, target: Some(format!("{table}.{}", use_.column)),
    }
}
pub(super) fn unknown(file: &str, line: usize) -> RuleFinding {
    RuleFinding { rule: RULE_ID.into(), file: file.into(), line,
        message: format!("{file}:{line}: cannot determine whether this predicate transforms a UUID column; use statically recoverable SQL and an unambiguous catalog-backed column reference, or set unanalyzableSql: ignore"),
        import: None, target: Some("unanalyzable".into()),
    }
}
