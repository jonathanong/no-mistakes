use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod effort_tests;
mod evaluate;
mod low;
#[cfg(test)]
mod policy_tests;
mod scan;
#[cfg(test)]
mod tests;

pub const RULE_ID: &str = "postgres-no-transformed-uuid-predicates";
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    sql_include_locked: bool,
    import_specifier: String,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    unanalyzable_sql: String,
    schema_catalog_path: String,
}
struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    schema_catalog_path: String,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
}
super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let observer = crate::diagnostics::current();
    let all: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .enumerate()
        .map(|(index, rule)| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let opts = compile_options(&rule.try_rule_options()?)?;
                let files = super::selected_rule_files(root, config, rule, all_files)?;
                let sql_paths =
                    crate::codebase::postgres::postgres_sql_paths(root, &files, &opts.schema)?;
                let sql_files = crate::codebase::check_facts::PathMembership::new(&sql_paths);
                let files = files
                    .into_iter()
                    .filter(|path| {
                        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
                        (opts.exclude.is_empty() || !opts.exclude.is_match(&rel))
                            && (sql_files.contains(path)
                                || (crate::codebase::dependencies::extract::is_indexable(path)
                                    && (opts.include.is_empty() || opts.include.is_match(&rel))))
                    })
                    .collect::<Vec<_>>();
                let mut findings = if config.effective_rule_analysis(rule)?.effort
                    == crate::config::v2::Effort::Low
                {
                    low::scan(root, &opts, &files, sources, facts, index, rule)?
                } else {
                    let view = facts.for_sql_application_with_advice(index, rule.name.as_deref());
                    scan::scan(root, &opts, &files, sources, &view)?
                };
                if let Some(message) = &rule.message {
                    for located in &mut findings.0 {
                        located.finding.message = format!("{message}: {}", located.finding.message);
                    }
                }
                Ok(findings)
            })
        })
        .collect();
    Ok(all?
        .into_iter()
        .flatten()
        .collect::<super::PostgresFindings>()
        .finish())
}
fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    crate::codebase::postgres::require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    GlobMatcher::new(&opts.sql_include, &format!("{RULE_ID} sqlInclude"))?;
    Ok(CompiledOptions {
        include: GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?,
        exclude: GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        schema_catalog_path: opts.schema_catalog_path.clone(),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
    })
}

#[cfg(test)]
mod cache_tests;

#[cfg(test)]
mod low_contract_tests;
