mod scc;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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
            let opts: Options = rule.rule_options();
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
    let package_names: HashSet<&str> = workspace
        .packages
        .iter()
        .map(|pkg| pkg.name.as_str())
        .collect();
    let dependency_types = dependency_types(opts);
    let allowlist: BTreeSet<String> = opts
        .allowlist
        .iter()
        .map(|cycle| canonical_cycle(cycle))
        .collect();
    let graph =
        workspace_graph_with_sources(&workspace, &package_names, &dependency_types, sources);

    let mut findings = Vec::new();
    for key in scc::cycle_keys(&graph) {
        if allowlist.contains(&key) {
            continue;
        }
        let cycle = display_cycle(&key);
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
    package_names: &HashSet<&str>,
    dependency_types: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph = BTreeMap::new();
    for package in &workspace.packages {
        let deps = package_dependencies_with_sources(
            &package.dir.join("package.json"),
            dependency_types,
            sources,
        )
        .into_iter()
        .filter(|dep| package_names.contains(dep.as_str()))
        .collect();
        graph.insert(package.name.clone(), deps);
    }
    graph
}

fn package_dependencies_with_sources(
    path: &Path,
    dependency_types: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> BTreeSet<String> {
    crate::codebase::package_deps::dependency_entries_from_source_store(
        path,
        dependency_types,
        sources,
    )
    .into_iter()
    .map(|entry| entry.name)
    .collect()
}

pub(super) fn canonical_cycle(cycle: &str) -> String {
    let mut nodes: Vec<String> = cycle
        .split("->")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    if nodes.first() == nodes.last() {
        nodes.pop();
    }
    if nodes.is_empty() {
        return String::new();
    }
    let min_idx = nodes
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    nodes.rotate_left(min_idx);
    nodes.join(" -> ")
}

fn display_cycle(canonical: &str) -> Vec<String> {
    let mut nodes: Vec<String> = canonical
        .split("->")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    if let Some(first) = nodes.first().cloned() {
        nodes.push(first);
    }
    nodes
}

#[cfg(test)]
mod tests;
