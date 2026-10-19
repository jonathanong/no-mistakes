use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) use super::github_actions_workflow_files::gha_kind;

mod checkout;
mod permissions;
mod uses;

pub const RULE_ID: &str = "github-actions-checkout-credentials";

/// The rule has no options. A non-empty options map is rejected.
#[derive(Debug, Default, Deserialize)]
struct Options;

super::sourced_file_rule_entrypoints!();

fn scan(
    root: &Path,
    _opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    super::github_actions_workflow_files::scan_files(root, files, sources, &|rel, content| {
        checkout::check_source(rel, content)
    })
}

#[cfg(test)]
mod tests;
