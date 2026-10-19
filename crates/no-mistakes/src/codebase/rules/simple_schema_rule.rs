use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::PostgresSchemaOptions;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
}

pub(crate) struct CompiledOptions {
    pub(crate) schema: PostgresSchemaOptions,
}

pub(crate) fn compile_options(opts: &Options) -> CompiledOptions {
    CompiledOptions {
        schema: super::schema_options(&opts.sql_include, opts.sql_include_locked),
    }
}

pub(crate) fn run_simple_schema_rule(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    facts: &CheckFactMap,
    rule_id: &str,
    scan: impl Fn(
        &Path,
        &CompiledOptions,
        &[PathBuf],
        &SourceStore,
        &CheckFactMap,
    ) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    super::run_prepared_file_rule(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        rule_id,
        |rule| Ok(compile_options(&rule.try_rule_options()?)),
        scan,
    )
}
