use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod compile;
mod compile_fk;
mod follow;
mod foreign;
mod require;
mod reserved;
mod scan;
mod singular;
mod types;

pub const RULE_ID: &str = "postgres-column-naming";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    pub(crate) type_rules: Vec<TypeRule>,
    pub(crate) name_type_rules: Vec<NameTypeRule>,
    pub(crate) skip_generated_columns: bool,
    pub(crate) ignore_table_patterns: Vec<String>,
    pub(crate) forbidden_column_names: Vec<ForbiddenName>,
    pub(crate) foreign_keys: ForeignKeyOptions,
    pub(crate) allow: Vec<ColumnAllowEntry>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ColumnAllowEntry {
    pub(crate) object: String,
    pub(crate) reason: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TypeRule {
    pub(crate) types: Vec<String>,
    pub(crate) name_pattern: String,
    pub(crate) hint: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NameTypeRule {
    pub(crate) name_pattern: String,
    pub(crate) types: Vec<String>,
    pub(crate) hint: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ForbiddenName {
    pub(crate) pattern: String,
    pub(crate) hint: String,
}

#[derive(Deserialize, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ForeignKeyOptions {
    pub(crate) target_suffixes: Vec<TargetSuffix>,
    pub(crate) reserved_suffixes: Vec<ReservedSuffix>,
    #[serde(default = "default_target_match")]
    pub(crate) target_match: String,
    pub(crate) target_names: Vec<TargetName>,
    pub(crate) check_self_references: bool,
    pub(crate) singular: BTreeMap<String, String>,
    pub(crate) follow_composite_foreign_keys: bool,
    pub(crate) require_foreign_key: Option<RequireForeignKey>,
}

impl Default for ForeignKeyOptions {
    fn default() -> Self {
        Self {
            target_suffixes: Vec::new(),
            reserved_suffixes: Vec::new(),
            target_match: default_target_match(),
            target_names: Vec::new(),
            check_self_references: false,
            singular: BTreeMap::new(),
            follow_composite_foreign_keys: false,
            require_foreign_key: None,
        }
    }
}

fn default_target_match() -> String {
    "off".to_string()
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TargetSuffix {
    pub(crate) tables: Vec<String>,
    pub(crate) suffixes: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReservedSuffix {
    pub(crate) suffix: String,
    pub(crate) types: Vec<String>,
    pub(crate) tables: Vec<String>,
    pub(crate) hint: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TargetName {
    pub(crate) table_pattern: String,
    pub(crate) name: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RequireForeignKey {
    pub(crate) types: Vec<String>,
    pub(crate) name_pattern: String,
    pub(crate) exempt: Vec<ExemptPattern>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExemptPattern {
    pub(crate) name_pattern: String,
    pub(crate) reason: String,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(config, &[RULE_ID])?;
    let facts = crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        Vec::new(),
        catalog_paths,
    );
    check_with_files_sources_and_facts(root, config, all_files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let compiled = compile::compile(&options, rule.message.clone())?;
        let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
        if !filter.is_match(Path::new(&options.schema_catalog_path)) {
            continue;
        }
        let catalog = facts.postgres_schema_catalog(&options.schema_catalog_path)?;
        findings.extend(scan::scan(catalog, &compiled, &options.schema_catalog_path));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
