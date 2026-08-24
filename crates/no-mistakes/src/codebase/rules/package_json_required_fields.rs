use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "package-json-required-fields";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) private: Option<bool>,
    #[serde(rename = "type")]
    pub(crate) type_value: Option<String>,
    pub(crate) license: Option<String>,
    pub(crate) require_scoped_name: bool,
    pub(crate) unscoped_name_exceptions: Vec<String>,
    pub(crate) main_when_file_exists: Option<String>,
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let in_scope: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let manifests = super::path_filter::filter_rule_files(root, config, rule, &in_scope)?;
        findings.extend(scan::scan(root, &opts, &manifests, &in_scope, sources));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn pkg_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
