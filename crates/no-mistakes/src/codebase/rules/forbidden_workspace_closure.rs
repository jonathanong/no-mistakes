use super::RuleFinding;
use crate::codebase::package_deps;
use crate::codebase::workspaces;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod alias;
mod lockfile;
mod traversal;

pub const RULE_ID: &str = "forbidden-workspace-closure";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) packages: Vec<String>,
    pub(crate) forbidden: Vec<String>,
    pub(crate) dependency_types: Vec<String>,
    pub(crate) lockfile: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Dependency {
    name: String,
    resolved_name: Option<String>,
    field: String,
}

#[derive(Debug, Clone)]
pub(super) struct PackageNode {
    manifest: PathBuf,
    deps: Vec<Dependency>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
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
            let source_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            scan(root, &target_roots, &opts, &files, &source_filter)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    target_roots: &[PathBuf],
    opts: &Options,
    files: &[PathBuf],
    source_filter: &super::path_filter::RulePathFilter,
) -> Result<Vec<RuleFinding>> {
    let workspace = load_workspace(root, target_roots, files)?;
    if workspace.packages.is_empty() {
        return Ok(Vec::new());
    }
    if opts.packages.is_empty() || opts.forbidden.is_empty() {
        return Ok(vec![config_finding(
            "each rule entry requires at least one packages entry and one forbidden entry",
        )]);
    }
    let forbidden = match build_globset(&opts.forbidden) {
        Ok(globs) => globs,
        Err(error) => {
            return Ok(vec![config_finding(&format!(
                "invalid glob pattern in forbidden: {error}"
            ))]);
        }
    };
    let dependency_types = dependency_types(opts);
    let mut nodes = manifest_nodes(&workspace, &dependency_types);
    if let Some(lockfile) = &opts.lockfile {
        match lockfile::lockfile_nodes(root, lockfile, &workspace, &nodes, &dependency_types) {
            Ok(lockfile_backed) => nodes = lockfile_backed,
            Err(message) => return Ok(vec![config_finding(&message)]),
        }
    }

    let workspace_names: BTreeSet<String> = nodes.keys().cloned().collect();
    let mut findings = Vec::new();
    for package in &opts.packages {
        if !nodes.contains_key(package) {
            findings.push(config_finding(&format!(
                "configured package '{package}' is not a named workspace package"
            )));
            continue;
        }
        traversal::collect_findings_for_package(
            root,
            package,
            &nodes,
            &workspace_names,
            &forbidden,
            source_filter,
            &mut findings,
        );
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn load_workspace(
    root: &Path,
    target_roots: &[PathBuf],
    files: &[PathBuf],
) -> Result<workspaces::WorkspaceMap> {
    let roots: Vec<&Path> = if target_roots.is_empty() {
        vec![root]
    } else {
        target_roots.iter().map(PathBuf::as_path).collect()
    };
    let mut packages = BTreeMap::new();
    for target_root in roots {
        for package in workspaces::load_from_files(target_root, files)?.packages {
            packages.insert(package.name.clone(), package);
        }
    }
    Ok(workspaces::WorkspaceMap {
        packages: packages.into_values().collect(),
    })
}

fn dependency_types(opts: &Options) -> Vec<&str> {
    if opts.dependency_types.is_empty() {
        package_deps::PRODUCTION_DEPENDENCY_FIELDS.to_vec()
    } else {
        opts.dependency_types.iter().map(String::as_str).collect()
    }
}

fn manifest_nodes(
    workspace: &workspaces::WorkspaceMap,
    dependency_types: &[&str],
) -> BTreeMap<String, PackageNode> {
    workspace
        .packages
        .iter()
        .map(|package| {
            let manifest = package.dir.join("package.json");
            let deps = package_deps::dependency_entries(&manifest, dependency_types)
                .into_iter()
                .map(|entry| Dependency {
                    name: entry.name,
                    resolved_name: alias::resolved_dependency_name(&entry.specifier),
                    field: entry.field,
                })
                .collect();
            (package.name.clone(), PackageNode { manifest, deps })
        })
        .collect()
}

fn build_globset(patterns: &[String]) -> std::result::Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    builder.build()
}

fn config_finding(message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: ".no-mistakes.yml".to_string(),
        line: 1,
        message: format!("{RULE_ID}: {message}"),
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_lockfile;
#[cfg(test)]
mod tests_lockfile_config;
