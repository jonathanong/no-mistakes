use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod comparison;
mod manifests;
mod scan;
pub(super) mod workspace;

pub const RULE_ID: &str = "package-json-nested-workspace-coverage";

const DEFAULT_DEPENDENCY_FIELDS: &[&str] =
    &["dependencies", "devDependencies", "optionalDependencies"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) roots: Vec<String>,
    pub(crate) dependency_name_prefixes: Vec<String>,
    pub(crate) dependency_fields: Vec<String>,
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
    scan::check_with_files_and_sources(root, config, all_files, sources)
}

#[cfg(test)]
mod tests;
