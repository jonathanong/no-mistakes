use crate::codebase::postgres::AllowEntry;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod compile;
mod scan;
mod text;

pub const RULE_ID: &str = "postgres-array-columns";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    #[serde(default = "default_never_allow")]
    pub(crate) never_allow_element_types: Vec<String>,
    pub(crate) allow_element_types: Vec<String>,
    pub(crate) allow_enum_elements: bool,
    pub(crate) allow: Vec<AllowEntry>,
}

fn default_never_allow() -> Vec<String> {
    vec!["uuid".to_string()]
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
