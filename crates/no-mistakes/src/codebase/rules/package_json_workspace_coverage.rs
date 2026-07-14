use super::RuleFinding;
use crate::codebase::glob_normalize;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files, sources)
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
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.package_roots.is_empty() {
        return Ok(Vec::new());
    }

    let workspace_dirs: HashSet<String> = workspaces::load_from_source_store(root, sources)?
        .packages
        .iter()
        .map(|pkg| relative_slash_path(root, &pkg.dir))
        .collect();
    let covered_workspace_dirs = covered_workspace_dirs(root, files, sources)?;
    let allowlist: HashSet<String> = opts
        .allowlist
        .iter()
        .map(|path| normalize_relative_path(path))
        .collect();

    let mut findings = Vec::new();
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            continue;
        }
        let rel = relative_slash_path(root, path);
        if allowlist.contains(rel.as_str()) {
            continue;
        }
        let dir = path
            .parent()
            .expect("discovered package.json paths have a parent directory");
        let dir_rel = relative_slash_path(root, dir);
        if dir_rel.is_empty() || !path_under_package_roots(&dir_rel, &opts.package_roots) {
            continue;
        }
        if workspace_dirs.contains(&dir_rel)
            || (!opts.require_named_package && covered_workspace_dirs.contains(&dir_rel))
        {
            continue;
        }
        if opts.require_named_package && package_name_with_sources(path, sources).is_none() {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: 1,
            message: format!("{rel}: package directory is not covered by the workspace config"),
            import: None,
            target: Some(dir_rel),
        });
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn covered_workspace_dirs(
    root: &Path,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<HashSet<String>> {
    let workspace_globs = workspaces::load_workspace_globs_from_source_store(root, sources)?;
    let include = build_workspace_globset(&workspace_globs, false)?;
    let exclude = build_workspace_globset(&workspace_globs, true)?;
    Ok(files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| path.parent())
        .filter_map(|dir| {
            let rel = relative_slash_path(root, dir);
            (include.is_match(rel.as_str()) && !exclude.is_match(rel.as_str())).then_some(rel)
        })
        .collect())
}

fn build_workspace_globset(patterns: &[String], excluded: bool) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let pattern = if excluded {
            let Some(stripped) = pattern.strip_prefix('!') else {
                continue;
            };
            stripped
        } else if pattern.starts_with('!') {
            continue;
        } else {
            pattern.as_str()
        };
        builder.add(Glob::new(&normalize_relative_path(pattern))?);
    }
    Ok(builder.build()?)
}

fn path_under_package_roots(path: &str, package_roots: &[String]) -> bool {
    package_roots.iter().any(|root| {
        let root = normalize_relative_path(root);
        !root.is_empty() && (path == root || path.starts_with(&format!("{root}/")))
    })
}

fn normalize_relative_path(path: &str) -> String {
    glob_normalize::normalize(path)
}

fn package_name_with_sources(
    path: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<String> {
    let json = sources.parse_json_path(path)?.ok()?;
    json.get("name")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests;
