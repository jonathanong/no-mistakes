use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod lockfile;
mod policy;
mod scan;

pub const RULE_ID: &str = "pnpm-release-age-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) permanent_packages: Vec<PermanentPackage>,
    pub(crate) temporary_selectors: Vec<String>,
    pub(crate) temporary_groups: Vec<TemporaryGroup>,
    pub(crate) scoped_prefixes: Vec<String>,
    pub(crate) workspace_yaml: Option<String>,
    pub(crate) dependabot_path: Option<String>,
    pub(crate) lockfile_path: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub(crate) struct PermanentPackage {
    pub(crate) name: String,
    pub(crate) reason: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct TemporaryGroup {
    pub(crate) selectors: Vec<String>,
    pub(crate) reason: String,
    pub(crate) eligible_for_removal_at: String,
}

impl Options {
    fn configured(&self) -> bool {
        !self.permanent_packages.is_empty()
            || !self.temporary_selectors.is_empty()
            || !self.temporary_groups.is_empty()
    }

    fn workspace_yaml(&self) -> &str {
        self.workspace_yaml
            .as_deref()
            .unwrap_or("pnpm-workspace.yaml")
    }

    fn dependabot_path(&self) -> &str {
        self.dependabot_path
            .as_deref()
            .unwrap_or(".github/dependabot.yml")
    }

    fn lockfile_path(&self) -> &str {
        self.lockfile_path.as_deref().unwrap_or("pnpm-lock.yaml")
    }
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
        if !opts.configured() {
            continue;
        }
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan::scan(root, &opts, &files, sources));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
