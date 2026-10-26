use super::{catalog, lockfile, options, selection, traversal, RuleFinding, RULE_ID};
use crate::codebase::package_deps;
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) struct ScanInput<'a> {
    pub(super) opts: &'a options::Options,
    pub(super) source_filter: &'a super::super::path_filter::RulePathFilter,
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) preparation: (
        &'a lockfile::LockfileImporterCache,
        &'a selection::SelectionCache,
    ),
    pub(super) anchor: &'a ConfigAnchor,
}

pub(super) fn scan_with_sources(
    root: &Path,
    target_roots: &[PathBuf],
    files: &[PathBuf],
    input: ScanInput<'_>,
) -> Result<Vec<RuleFinding>> {
    let ScanInput {
        opts,
        source_filter,
        sources,
        preparation,
        anchor,
    } = input;
    let (lockfile_importers, selections) = preparation;
    let selection = selections.get_or_compute(root, target_roots, files, sources)?;
    let workspace = &selection.workspace;
    if workspace.packages.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(finding) = missing_lists(anchor, opts) {
        return Ok(vec![finding]);
    }
    let forbidden = match build_globset(&opts.forbidden) {
        Ok(globs) => globs,
        Err(error) => {
            return Ok(vec![config_finding(
                anchor,
                anchor.invalid_glob_line("forbidden", &opts.forbidden),
                &format!("invalid glob pattern in forbidden: {error}"),
            )]);
        }
    };
    let dependency_types = match options::dependency_types(opts) {
        Ok(dependency_types) => dependency_types,
        Err(message) => return Ok(vec![dependency_type_finding(anchor, opts, &message)]),
    };
    // Lockfile field matching and manifest-only peers need every field;
    // the selected-field projection would be discarded on that path.
    let manifest_nodes = selection.manifest_nodes(
        root,
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
            Err(message) => {
                return Ok(vec![config_finding(
                    anchor,
                    anchor.option_line("lockfile"),
                    &message,
                )])
            }
        };
        &lockfile_nodes
    } else {
        manifest_nodes.as_ref()
    };

    let workspace_names: BTreeSet<String> = nodes.keys().cloned().collect();
    let mut findings = catalog::duplicate_setting_findings(root, sources, source_filter);
    let traversal = traversal::Traversal::new(
        root,
        nodes,
        &workspace_names,
        &forbidden,
        source_filter,
        sources.observer().map(std::convert::AsRef::as_ref),
    );
    for package in &opts.packages {
        if !nodes.contains_key(package) {
            findings.push(config_finding(
                anchor,
                anchor.listed_line("packages", &opts.packages, package),
                &format!("configured package '{package}' is not a named workspace package"),
            ));
            continue;
        }
        traversal.collect_findings_for_package(package, &mut findings);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn missing_lists(anchor: &ConfigAnchor, opts: &options::Options) -> Option<RuleFinding> {
    if !opts.packages.is_empty() && !opts.forbidden.is_empty() {
        return None;
    }
    let key = if opts.packages.is_empty() {
        "packages"
    } else {
        "forbidden"
    };
    Some(config_finding(
        anchor,
        anchor.option_line(key),
        "each rule entry requires at least one packages entry and one forbidden entry",
    ))
}

fn dependency_type_finding(
    anchor: &ConfigAnchor,
    opts: &options::Options,
    message: &str,
) -> RuleFinding {
    let line = opts
        .dependency_types
        .iter()
        .position(|field| !package_deps::ALL_DEPENDENCY_FIELDS.contains(&field.as_str()))
        .map(|index| anchor.option_item_line("dependencyTypes", index))
        .unwrap_or_else(|| anchor.option_line("dependencyTypes"));
    config_finding(anchor, line, message)
}

pub(super) fn build_globset(patterns: &[String]) -> std::result::Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    builder.build()
}

fn config_finding(anchor: &ConfigAnchor, line: usize, message: &str) -> RuleFinding {
    anchor.emit(RULE_ID, line, format!("{RULE_ID}: {message}"), None, None)
}
