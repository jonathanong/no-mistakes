use super::Options;
use crate::codebase::postgres::{require_catalog_path, AllowList};
use anyhow::{bail, Result};
use regex::Regex;

pub(super) struct Compiled {
    pub(super) types: Vec<String>,
    pub(super) names: Vec<Regex>,
    pub(super) name_raw: Vec<String>,
    pub(super) skip_generated: bool,
    pub(super) ignore: Vec<Regex>,
    pub(super) allow: AllowList,
}

impl Compiled {
    pub(super) fn type_matches(&self, data_type: &str) -> bool {
        self.types
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(data_type))
    }

    pub(super) fn matching_pattern(&self, name: &str) -> Option<&str> {
        self.names
            .iter()
            .zip(&self.name_raw)
            .find(|(pattern, _)| pattern.is_match(name))
            .map(|(_, raw)| raw.as_str())
    }

    pub(super) fn ignores(&self, table: &str) -> bool {
        self.ignore.iter().any(|pattern| pattern.is_match(table))
    }
}

pub(super) fn compile(options: &Options) -> Result<Compiled> {
    require_catalog_path(super::RULE_ID, options.schema_catalog_path.trim())?;
    if options.column_types.is_empty() {
        bail!("{} option columnTypes: empty", super::RULE_ID);
    }
    Ok(Compiled {
        types: options.column_types.clone(),
        names: patterns("namePatterns", &options.name_patterns)?,
        name_raw: options.name_patterns.clone(),
        skip_generated: options.skip_generated_columns,
        ignore: patterns("ignoreTablePatterns", &options.ignore_table_patterns)?,
        allow: AllowList::compile(super::RULE_ID, options.allow.clone())?,
    })
}

fn patterns(option: &str, values: &[String]) -> Result<Vec<Regex>> {
    values
        .iter()
        .map(|pattern| {
            Regex::new(pattern).map_err(|error| {
                anyhow::anyhow!("{} option {option}: invalid regex: {error}", super::RULE_ID)
            })
        })
        .collect()
}
