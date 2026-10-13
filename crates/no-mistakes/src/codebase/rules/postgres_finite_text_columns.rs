use crate::codebase::postgres::AllowEntry;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod compile;
mod parse;
mod pin;
mod scan;
mod text;

pub const RULE_ID: &str = "postgres-finite-text-columns";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    #[serde(default = "default_column_types")]
    pub(crate) column_types: Vec<String>,
    pub(crate) name_patterns: Vec<String>,
    pub(crate) skip_generated_columns: bool,
    pub(crate) ignore_table_patterns: Vec<String>,
    pub(crate) allow: Vec<AllowEntry>,
}

fn default_column_types() -> Vec<String> {
    vec!["text".to_string(), "character varying".to_string()]
}

super::prepared_rule_entrypoints!(super::check_prepared_schema_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_catalog_rule(
        root,
        config,
        facts,
        RULE_ID,
        |rule| {
            let options: Options = rule.try_rule_options()?;
            let compiled = compile::compile(&options, rule.message.clone())?;
            Ok((options.schema_catalog_path, compiled))
        },
        |compiled, catalog, path| scan::scan(catalog, &compiled, path),
    )
}

#[cfg(test)]
mod tests;
