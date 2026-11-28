use crate::config::v2::{
    schema::PostgresPolymorphicTargetOptions as Options, NoMistakesConfig,
};
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use std::path::{Path, PathBuf};

pub(in crate::codebase::rules) mod compile;
mod scan;

pub const RULE_ID: &str = "postgres-polymorphic-target";

super::prepared_rule_entrypoints!(super::check_prepared_schema_rule, compile_applications);

fn compile_applications(config: &NoMistakesConfig) -> Result<()> {
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        compile::compile(&options, rule.message.clone())?;
    }
    Ok(())
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, RULE_ID, |rule| {
        let options: Options = rule.try_rule_options()?;
        let compiled = compile::compile(&options, rule.message.clone())?;
        let Some(catalog) = super::selected_schema_catalog(
            root,
            config,
            rule,
            facts,
            &options.schema_catalog_path,
        )?
        else {
            return Ok(Vec::new());
        };
        Ok(scan::scan(compiled, catalog, &options.schema_catalog_path))
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
