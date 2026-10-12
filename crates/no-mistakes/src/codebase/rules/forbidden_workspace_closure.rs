use super::RuleFinding;
use crate::codebase::package_deps;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod alias;
mod lockfile;
mod manifest;
mod options;
mod selection;
mod traversal;
mod workspace;
use options::Options;

pub const RULE_ID: &str = "forbidden-workspace-closure";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Dependency {
    name: String,
    resolved_name: Option<String>,
    workspace_name: Option<String>,
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
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let lockfile_importers =
        lockfile::LockfileImporterCache::with_observer(sources.observer().cloned());
    let selections = selection::SelectionCache::default();
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let mut discovery_roots = vec![root.to_path_buf()];
            discovery_roots.extend(target_roots.iter().cloned());
            discovery_roots.sort();
            discovery_roots.dedup();
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &discovery_roots))
                .cloned()
                .collect();
            let source_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            scan_with_sources(
                root,
                &target_roots,
                &opts,
                &files,
                &source_filter,
                sources,
                (&lockfile_importers, &selections),
            )
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_sources(
    root: &Path,
    target_roots: &[PathBuf],
    opts: &Options,
    files: &[PathBuf],
    source_filter: &super::path_filter::RulePathFilter,
    sources: &crate::codebase::ts_source::SourceStore,
    preparation: (&lockfile::LockfileImporterCache, &selection::SelectionCache),
) -> Result<Vec<RuleFinding>> {
    let (lockfile_importers, selections) = preparation;
    let selection = selections.get_or_compute(root, target_roots, files, sources)?;
    let workspace = &selection.workspace;
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
    let dependency_types = match options::dependency_types(opts) {
        Ok(dependency_types) => dependency_types,
        Err(message) => return Ok(vec![config_finding(&message)]),
    };
    // Lockfile field matching and manifest-only peers need every field;
    // the selected-field projection would be discarded on that path.
    let manifest_nodes = selection.manifest_nodes(
        if opts.lockfile.is_some() {
            package_deps::ALL_DEPENDENCY_FIELDS
        } else {
            &dependency_types
        },
        sources,
    );
    let lockfile_nodes;
    let nodes = if let Some(lockfile) = &opts.lockfile {
        let lockfile_base = lockfile::base_root(root, target_roots);
        lockfile_nodes = match lockfile::lockfile_nodes(
            root,
            lockfile_base,
            lockfile,
            workspace,
            &manifest_nodes,
            lockfile_importers,
            (&dependency_types, &opts.packages, sources),
        ) {
            Ok(lockfile_backed) => lockfile_backed,
            Err(message) => return Ok(vec![config_finding(&message)]),
        };
        &lockfile_nodes
    } else {
        manifest_nodes.as_ref()
    };

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
            nodes,
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
mod tests_lockfile_alias;
#[cfg(test)]
mod tests_lockfile_config;
#[cfg(test)]
mod tests_lockfile_path_alias;
#[cfg(test)]
mod tests_manifest_alias;
#[cfg(test)]
mod tests_manifest_path_alias;
#[cfg(test)]
mod tests_pnpm12;
#[cfg(test)]
mod tests_workspace;
#[cfg(test)]
mod tests_workspace_lockfile;
