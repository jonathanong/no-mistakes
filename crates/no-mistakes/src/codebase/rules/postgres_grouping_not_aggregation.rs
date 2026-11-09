use super::{path_filter::GlobMatcher, RuleFinding};
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};
mod analyze;
mod candidate;
mod compile;
mod evidence;
mod existence;
mod expressions;
mod filters;
mod scan;
mod scope;
mod union;
mod unique;
mod walk;
pub const RULE_ID: &str = "postgres-grouping-not-aggregation";
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    schema_catalog_path: String,
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    sql_include_locked: bool,
    import_specifier: String,
    executor_names: Vec<String>,
    extra_executor_names: Option<Vec<String>>,
    exclude_executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    executor_option_properties: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    sql_literal_helpers: Vec<String>,
}
struct CompiledOptions {
    schema_catalog_path: String,
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
}
super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);
pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let results: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            let opts = compile::options(&opts)?;
            if opts.schema_catalog_path.trim().is_empty() {
                return Ok(super::PostgresFindings::default());
            }
            let files = super::selected_rule_files(root, config, rule, all_files)?;
            let sql_paths =
                crate::codebase::postgres::postgres_sql_paths(root, &files, &opts.schema)?;
            let sql = crate::codebase::check_facts::PathMembership::new(&sql_paths);
            let files = files
                .into_iter()
                .filter(|path| {
                    let rel = relative_slash_path(root, path);
                    (opts.exclude.is_empty() || !opts.exclude.is_match(&rel))
                        && if rel.ends_with(".sql") {
                            sql.contains(path)
                        } else {
                            crate::codebase::dependencies::extract::is_indexable(path)
                                && (opts.include.is_empty() || opts.include.is_match(&rel))
                        }
                })
                .collect::<Vec<_>>();
            scan::scan(root, &opts, &files, sources, facts)
        })
        .collect();
    Ok(results?
        .into_iter()
        .flatten()
        .collect::<super::PostgresFindings>()
        .finish())
}

#[cfg(test)]
mod review_tests;

#[cfg(test)]
mod recovery_tests;
