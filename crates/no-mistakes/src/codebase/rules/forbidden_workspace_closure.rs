use super::RuleFinding;
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod alias;
mod catalog;
mod lockfile;
mod manifest;
mod options;
mod scan;
mod selection;
mod traversal;
mod workspace;

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
            let opts: options::Options = rule.try_rule_options()?;
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
            let anchor = ConfigAnchor::resolve(root, config, Some(sources), rule);
            scan::scan_with_sources(
                root,
                &target_roots,
                &files,
                scan::ScanInput {
                    opts: &opts,
                    source_filter: &source_filter,
                    sources,
                    preparation: (&lockfile_importers, &selections),
                    anchor: &anchor,
                },
            )
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

include!("forbidden_workspace_closure/test_mods.rs");
