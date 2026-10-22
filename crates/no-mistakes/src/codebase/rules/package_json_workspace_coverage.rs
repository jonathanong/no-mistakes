use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod coverage_scope;
mod patterns;
#[cfg(test)]
use coverage_scope::package_name_with_sources;
use patterns::{covered_workspace_dirs, normalize_relative_path};

pub const RULE_ID: &str = "package-json-workspace-coverage";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) package_roots: Vec<String>,
    pub(crate) allowlist: Vec<String>,
    pub(crate) require_named_package: bool,
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
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files, all_files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.package_roots.is_empty() {
        return Ok(Vec::new());
    }

    let coverage_scope::WorkspaceCoverage {
        mut findings,
        workspace_globs,
        workspace_dirs,
    } = coverage_scope::resolve_workspace(root, files, all_files, sources)?;
    let covered_workspace_dirs = covered_workspace_dirs(root, files, &workspace_globs)?;
    let allowlist: HashSet<String> = opts
        .allowlist
        .iter()
        .map(|path| normalize_relative_path(path))
        .collect();

    for path in files {
        findings.extend(coverage_scope::package_findings(
            root,
            opts,
            path,
            sources,
            &allowlist,
            &workspace_dirs,
            &covered_workspace_dirs,
        ));
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

#[cfg(test)]
mod tests;
