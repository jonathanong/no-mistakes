use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) use super::github_actions_workflow_files::gha_kind;

mod checkout;
mod permissions;
mod uses;

pub const RULE_ID: &str = "github-actions-checkout-credentials";

/// `assumeWriteWhenUnspecified` defaults to false. Unknown option keys are rejected.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    assume_write_when_unspecified: bool,
}

super::sourced_file_rule_entrypoints!();

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let assume = opts.assume_write_when_unspecified;
    super::github_actions_workflow_files::scan_files(root, files, sources, &|rel, content| {
        checkout::check_source(rel, content, assume)
    })
}

#[cfg(test)]
mod tests;
