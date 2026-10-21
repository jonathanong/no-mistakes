use super::RuleFinding;
use crate::codebase::postgres::{
    AllowEntry, AllowList, EmbeddedSqlOptions, PostgresSchemaOptions, SqlBoundKind,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod arrays;
mod evaluate;
mod scan;

pub const RULE_ID: &str = "postgres-bounded-statements";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    unanalyzable_sql: String,
    schema_catalog_path: String,
    statements: Option<Vec<String>>,
    allow: Vec<AllowEntry>,
}

pub(crate) struct CompiledOptions {
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
    schema_catalog_path: String,
    statements: Vec<SqlBoundKind>,
    allow: AllowList,
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_sql_rule_with_glob_filters(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        compile_options,
        |opts: &Options| (&opts.include, &opts.exclude),
        |root, compiled, files, sources, facts| {
            scan::scan(root, compiled, files, sources, Some(facts))
        },
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    crate::codebase::postgres::require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
        schema_catalog_path: opts.schema_catalog_path.clone(),
        statements: statement_kinds(opts.statements.as_deref())?,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
    })
}

fn statement_kinds(configured: Option<&[String]>) -> Result<Vec<SqlBoundKind>> {
    let Some(configured) = configured else {
        return Ok(vec![
            SqlBoundKind::Select,
            SqlBoundKind::Update,
            SqlBoundKind::Delete,
        ]);
    };
    if configured.is_empty() {
        bail!("{RULE_ID} option statements: must name at least one of select, update, delete");
    }
    let mut kinds = Vec::new();
    for name in configured {
        let kind = match name.trim().to_ascii_lowercase().as_str() {
            "select" => SqlBoundKind::Select,
            "update" => SqlBoundKind::Update,
            "delete" => SqlBoundKind::Delete,
            _ => bail!("{RULE_ID} option statements: unknown statement {name}; use select, update or delete"),
        };
        if kinds.contains(&kind) {
            bail!("{RULE_ID} option statements: duplicate entry {name}");
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;

/// Unstable adapter for the prepared evaluator's Criterion harness.
#[cfg(feature = "test-instrumentation")]
pub(crate) fn benchmark_offenders(
    facts: &crate::codebase::postgres::SqlStatementFileFacts,
    catalog: &crate::codebase::postgres::SchemaCatalog,
) -> usize {
    facts
        .bounds
        .iter()
        .map(|fact| evaluate::offenders(fact, catalog).len())
        .sum()
}
