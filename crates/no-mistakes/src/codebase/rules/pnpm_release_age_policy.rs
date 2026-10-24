use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod included_exclude;
mod inputs;
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

super::borrowed_source_rule_entrypoints!();
mod effort;
pub(crate) use effort::check_with_files_sources_and_deferred_suppression;
pub use effort::diagnostics_with_files_and_sources;

fn rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod scan_edges;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_include;
#[cfg(test)]
mod tests_pnpm12;
