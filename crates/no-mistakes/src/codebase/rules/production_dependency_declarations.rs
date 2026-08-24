//! Reject production imports of a package the owning `package.json` does not
//! declare under an allowed dependency field (`dependencies`,
//! `optionalDependencies`, `peerDependencies` by default).
//!
//! Local pnpm linking and Node's directory walk-up resolve an undeclared or
//! `devDependencies`-only import fine in development. A filtered
//! `pnpm deploy --prod` install prunes `devDependencies` and relocates
//! workspace packages, so the same import fails at runtime in production.
//! See `docs/rules/production-dependency-declarations.md`.

mod discovery;
mod findings;
mod manifest;
mod reachability;
mod scan;
mod specifier;

use super::RuleFinding;
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "production-dependency-declarations";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) workspace_roots: Vec<String>,
    pub(crate) allowed_fields: Vec<String>,
    pub(crate) test_file_patterns: Vec<String>,
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
            let opts: Options = rule.try_rule_options()?;
            let workspace_roots = match workspace_roots(root, &opts) {
                Ok(roots) => roots,
                Err(message) => return Ok(vec![findings::config(&message)]),
            };
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &workspace_roots))
                .cloned()
                .collect();
            scan::run(root, &workspace_roots, &opts, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn workspace_roots(root: &Path, opts: &Options) -> Result<Vec<PathBuf>, String> {
    if opts.workspace_roots.is_empty() {
        return Err(
            "workspaceRoots is required and must list at least one workspace root; it is not \
             inferred from the check root"
                .to_string(),
        );
    }
    Ok(opts
        .workspace_roots
        .iter()
        .map(|relative| normalize_path(&root.join(relative)))
        .collect())
}

#[cfg(test)]
mod tests;
