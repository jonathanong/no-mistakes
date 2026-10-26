use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) use super::github_actions_workflow_files::gha_kind;

mod checkout;
mod diagnostics;
mod events;
mod findings;
mod inputs;
mod permissions;
mod refs;
mod uses;

pub const RULE_ID: &str = "github-actions-checkout-credentials";

/// `assumeWriteWhenUnspecified` defaults to false. Unknown option keys are rejected.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    assume_write_when_unspecified: bool,
    default_branch: Option<String>,
}

impl Options {
    fn default_branch(&self) -> Result<Option<&str>> {
        let Some(branch) = self.default_branch.as_deref() else {
            return Ok(None);
        };
        let branch = branch
            .trim()
            .strip_prefix("refs/heads/")
            .unwrap_or(branch.trim());
        anyhow::ensure!(
            !branch.is_empty() && !branch.contains("${{")
                && branch != "refs/pull" && !branch.starts_with("refs/pull/"),
            "{RULE_ID}: options.defaultBranch must be a nonempty literal base branch name, not an expression or PR ref"
        );
        Ok(Some(branch))
    }
}

pub fn check(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, |root, config, files| {
        super::check_with_borrowed_source_store(
            root,
            config,
            files,
            |root, config, files, sources| {
                diagnostics_with_files_and_sources(root, config, files, sources, false)
                    .map(|output| output.findings)
            },
        )
    })
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    // Preserve the file-rule boundary: its caller owns suppression.
    diagnostics_with_files_and_sources(root, config, files, sources, true)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub use diagnostics::run as diagnostics_with_files_and_sources;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "github_actions_checkout_credentials/tests/policy.rs"]
mod policy_tests;

#[cfg(test)]
#[path = "github_actions_checkout_credentials/tests/permission_bounds.rs"]
mod permission_bounds_tests;

#[cfg(test)]
#[path = "github_actions_checkout_credentials/tests/ref_safety.rs"]
mod ref_safety_tests;

#[cfg(test)]
#[path = "github_actions_checkout_credentials/tests/analysis.rs"]
mod analysis_tests;
