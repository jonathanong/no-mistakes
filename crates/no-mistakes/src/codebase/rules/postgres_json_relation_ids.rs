use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod compile;
mod fingerprint;
mod scan;

pub const RULE_ID: &str = "postgres-json-relation-ids";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    pub(crate) id_key_pattern: Option<String>,
    pub(crate) key_column_map: BTreeMap<String, String>,
    pub(crate) fingerprint_name_pattern: Option<String>,
    pub(crate) allow: Vec<crate::codebase::postgres::AllowEntry>,
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) sql_include_locked: bool,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
}

struct CompiledOptions {
    schema_catalog_path: String,
    id_key_pattern: Option<Regex>,
    key_column_map: BTreeMap<String, String>,
    fingerprint_name_pattern: Option<Regex>,
    allow: BTreeSet<String>,
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
}

impl CompiledOptions {
    fn allows(&self, table: &str, column: &str) -> bool {
        self.allow.contains(&format!("table:{table}"))
            || self.allow.contains(&format!("column:{table}.{column}"))
    }

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
    let all: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            let compiled = compile::compile_options(&opts)?;
            if compiled.id_key_pattern.is_none() && compiled.fingerprint_name_pattern.is_none() {
                return Ok(super::PostgresFindings::default());
            }
            let files = super::selected_rule_files(root, config, rule, all_files)?;
            let sql_paths =
                crate::codebase::postgres::postgres_sql_paths(root, &files, &compiled.schema)?;
            let sql = crate::codebase::check_facts::PathMembership::new(&sql_paths);
            let files = files
                .into_iter()
                .filter(|path| {
                    compiled.includes(&relative_slash_path(root, path), sql.contains(path))
                })
                .collect::<Vec<_>>();
            scan::scan(root, &compiled, &files, sources, facts)
        })
        .collect();
    let findings: super::PostgresFindings = all?.into_iter().flatten().collect();
    Ok(findings.finish())
}

#[cfg(test)]
mod embedded_tests;
#[cfg(test)]
mod fingerprint_expression_tests;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod recall_tests;
#[cfg(test)]
mod tests;
