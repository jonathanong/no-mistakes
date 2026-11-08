use super::RuleFinding;
use crate::config::v2::{
    schema::PostgresColumnCheckConstraintsOptions as Options, NoMistakesConfig,
};
use anyhow::Result;
use std::path::{Path, PathBuf};

mod compile;
mod proof;
mod scan;

pub const RULE_ID: &str = "postgres-column-check-constraints";

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
        if options.schema_catalog_path.is_empty() {
            return Ok(Vec::new());
        }
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

#[cfg(test)]
mod tests;
