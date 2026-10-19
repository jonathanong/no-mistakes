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
    /// Copied from `projects.<name>.postgres` when this rule references that project.
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) import_specifier: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) executor_factory_names: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) executor_type_names: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) unanalyzable_sql: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) sql_include_locked: bool,
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
    pub(crate) allow_referenced_column_name: bool,
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
            allow_referenced_column_name: false,
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
