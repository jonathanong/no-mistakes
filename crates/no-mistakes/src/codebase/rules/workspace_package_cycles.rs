mod edges;
mod range;
mod scc;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "workspace-package-cycles";

const DEFAULT_DEPENDENCY_TYPES: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) dependency_types: Vec<String>,
    pub(crate) allowlist: Vec<String>,
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
            scan_with_sources(root, &opts, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_sources(
    root: &Path,
    opts: &Options,
    _files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let workspace = workspaces::load_from_source_store(root, sources)?;
    if workspace.packages.is_empty() {
        return Ok(Vec::new());
    }

    let package_dirs: HashMap<String, PathBuf> = workspace
        .packages
        .iter()
        .map(|pkg| (pkg.name.clone(), pkg.dir.clone()))
        .collect();
    let dependency_types = dependency_types(opts);
    let allowlist: BTreeSet<String> = opts
        .allowlist
        .iter()
        .map(|cycle| scc::canonical_cycle(cycle))
        .collect();
    let graph = workspace_graph_with_sources(&workspace, &dependency_types, sources);

    let mut findings = Vec::new();
    for key in scc::cycle_keys(&graph) {
        if allowlist.contains(&key) {
            continue;
        }
        let cycle = scc::display_cycle(&key);
        let first = cycle
            .first()
            .expect("collected cycle keys always contain at least one package");
        let dir = package_dirs
            .get(first)
            .expect("cycle packages are collected from the workspace graph");
        let file = relative_slash_path(root, &dir.join("package.json"));
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: 1,
            message: format!(
                "{file}: workspace package cycle detected: {}",
                cycle.join(" -> ")
            ),
            import: None,
            target: Some(cycle.join(" -> ")),
        });
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn dependency_types(opts: &Options) -> Vec<&str> {
    if opts.dependency_types.is_empty() {
        DEFAULT_DEPENDENCY_TYPES.to_vec()
    } else {
        opts.dependency_types.iter().map(String::as_str).collect()
    }
}

fn workspace_graph_with_sources(
    workspace: &workspaces::WorkspaceMap,
    dependency_types: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph = BTreeMap::new();
    for package in &workspace.packages {
        let deps =
            edges::workspace_dependency_names(&package.dir, dependency_types, workspace, sources);
        graph.insert(package.name.clone(), deps);
    }
    graph
}

#[cfg(test)]
mod tests;
