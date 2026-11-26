use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{fail_unanalyzable_sql, EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-no-duplicate-keyed-lookups";

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) sql_include_locked: bool,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) extra_executor_names: Option<Vec<String>>,
    pub(crate) exclude_executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) executor_option_properties: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) unanalyzable_sql: String,
}

pub(in crate::codebase::rules) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    pub(super) schema: PostgresSchemaOptions,
    pub(super) embedded: EmbeddedSqlOptions,
    pub(super) fail_unanalyzable: bool,
    pub(super) advise_unanalyzable: bool,
}

impl CompiledOptions {
    fn includes(&self, rel: &str, sql: bool) -> bool {
        if !self.exclude.is_empty() && self.exclude.is_match(rel) {
            return false;
        }
        if rel.ends_with(".sql") {
            return sql;
        }
        crate::codebase::dependencies::extract::is_indexable(Path::new(rel))
            && (self.include.is_empty() || self.include.is_match(rel))
    }
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let observer = crate::diagnostics::current();
    let all: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .enumerate()
        .map(|(index, rule)| {
            crate::diagnostics::with_observer(observer.clone(), || {
                let opts: Options = rule.try_rule_options()?;
                let compiled = compile_options(&opts)?;
                let files = super::selected_rule_files(root, config, rule, all_files)?;
                let sql_paths =
                    crate::codebase::postgres::postgres_sql_paths(root, &files, &compiled.schema)?;
                let sql = crate::codebase::check_facts::PathMembership::new(&sql_paths);
                let files: Vec<PathBuf> = files
                    .into_iter()
                    .filter(|path| {
                        compiled.includes(&relative_slash_path(root, path), sql.contains(path))
                    })
                    .collect();
                let view = facts.for_sql_application_with_advice(index, rule.name.as_deref());
                let mut findings =
                    scan::scan(root, &compiled, &files, &view as &dyn SqlFactContext, sources)?;
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

pub(in crate::codebase::rules) fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    GlobMatcher::new(&opts.sql_include, &format!("{RULE_ID} sqlInclude"))?;
    let sql_include = if opts.sql_include_locked || !opts.sql_include.is_empty() {
        opts.sql_include.clone()
    } else {
        Vec::new()
    };
    Ok(CompiledOptions {
        include,
        exclude,
        schema: PostgresSchemaOptions { sql_include },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_extra_executor_names(opts.extra_executor_names.as_deref())
            .with_exclude_executor_names(&opts.exclude_executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_executor_option_properties(&opts.executor_option_properties)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: fail_unanalyzable_sql(RULE_ID, &opts.unanalyzable_sql)?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
