use super::RuleFinding;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-require-query-annotation";

use scan::scan_with_sources;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) unanalyzable_sql: UnanalyzableSql,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum UnanalyzableSql {
    #[default]
    Report,
    Ignore,
    Advise,
}

struct CompiledOptions {
    paths: super::RuleGlobFilter,
    embedded: EmbeddedSqlOptions,
    unanalyzable_sql: UnanalyzableSql,
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_included_postgres_rule(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        |rule| compile_options(&rule.try_rule_options()?),
        |compiled, rel| compiled.paths.includes(rel),
        scan_with_sources,
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        paths: super::RuleGlobFilter::new(RULE_ID, &opts.include, &opts.exclude)?,
        unanalyzable_sql: opts.unanalyzable_sql,
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
    })
}

#[cfg(test)]
mod nested_fragment_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict_tests;
