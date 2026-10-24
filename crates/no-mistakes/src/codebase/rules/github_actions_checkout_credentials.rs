use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) use super::github_actions_workflow_files::gha_kind;

mod checkout;
mod diagnostics;
mod inputs;
mod permissions;
mod uses;

pub const RULE_ID: &str = "github-actions-checkout-credentials";

/// `assumeWriteWhenUnspecified` defaults to false. Unknown option keys are rejected.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    assume_write_when_unspecified: bool,
}

pub fn check(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
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
