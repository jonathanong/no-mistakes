use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SchemaCatalog;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::path::Path;

/// Compile each application before path filtering, then read its prepared catalog.
pub(crate) fn run_catalog_rule<C>(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
    rule_id: &str,
    compile: impl Fn(&RuleDef) -> Result<(String, C)>,
    scan: impl Fn(C, &SchemaCatalog, &str) -> Vec<RuleFinding>,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, rule_id, |rule| {
        let (path, compiled) = compile(rule)?;
        let Some(catalog) = super::selected_schema_catalog(root, config, rule, facts, &path)?
        else {
            return Ok(Vec::new());
        };
        Ok(scan(compiled, catalog, &path))
    })
}
