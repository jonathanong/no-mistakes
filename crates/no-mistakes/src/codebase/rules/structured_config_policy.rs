use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

mod equals_file;
mod scan;
mod value_assertions;
mod when;
use scan::scan;
use value_assertions::assert_value;

pub const RULE_ID: &str = "structured-config-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) policies: Vec<Policy>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Policy {
    pub(crate) files: Vec<String>,
    pub(crate) required_keys: Vec<String>,
    pub(crate) banned_keys: Vec<String>,
    pub(crate) value_assertions: Vec<ValueAssertion>,
    pub(crate) when: Vec<PolicyWhen>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PolicyWhen {
    pub(crate) key: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ValueAssertion {
    pub(crate) key: String,
    #[serde(default, deserialize_with = "deserialize_assertion_kind")]
    pub(crate) kind: Option<AssertionKind>,
    pub(crate) prefix: String,
    pub(crate) glob: String,
    pub(crate) value: Option<Value>,
    pub(crate) required_keys: Vec<String>,
    pub(crate) forbidden_keys: Vec<String>,
    pub(crate) required_values: std::collections::BTreeMap<String, Value>,
    pub(crate) message: Option<String>,
    pub(crate) file: String,
    pub(crate) from_key: String,
    #[serde(rename = "match", default)]
    pub(crate) match_mode: MatchMode,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MatchMode {
    #[default]
    All,
    Any,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AssertionKind {
    Boolean,
    RecordOfBoolean,
    PositiveNumber,
    StringArray,
    StringPrefix,
    StringGlob,
    NotSingleFile,
    Equals,
    EqualsFile,
    ObjectShape,
}

impl AssertionKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "boolean" => Some(Self::Boolean),
            "record-of-boolean" => Some(Self::RecordOfBoolean),
            "positive-number" => Some(Self::PositiveNumber),
            "string-array" => Some(Self::StringArray),
            "string-prefix" => Some(Self::StringPrefix),
            "string-glob" => Some(Self::StringGlob),
            "not-single-file" => Some(Self::NotSingleFile),
            "equals" => Some(Self::Equals),
            "equals-file" => Some(Self::EqualsFile),
            "object-shape" => Some(Self::ObjectShape),
            _ => None,
        }
    }
}

fn deserialize_assertion_kind<'de, D>(deserializer: D) -> Result<Option<AssertionKind>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.as_deref().and_then(AssertionKind::from_str))
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
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files, &target_roots, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn value_at_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('.')
        .try_fold(value, |current, part| current.get(part))
}

#[cfg(test)]
mod bind_tests;
#[cfg(test)]
mod equals_file_tests;
#[cfg(test)]
mod jsonc_tests;
#[cfg(test)]
mod plugins_tests;
#[cfg(test)]
mod selector_tests;
#[cfg(test)]
mod tests;
