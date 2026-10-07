use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod config;
mod extract;
mod options;
mod routes;
mod scan;
mod static_values;

use options::Options;
use scan::scan;

pub const RULE_ID: &str = "nextjs-redirect-destinations";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &snapshot.paths_for(root),
    );
    let sources = snapshot.source_store_for(root);
    check_with_files_sources_and_snapshot(root, config, &files, &sources, Some(&snapshot))
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
    check_with_files_sources_and_snapshot(root, config, all_files, sources, None)
}

pub(crate) fn check_with_files_sources_and_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
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
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let findings = if opts.tracked_routes_only {
                let snapshot = snapshot.ok_or_else(|| anyhow::anyhow!(
                    "nextjs-redirect-destinations trackedRoutesOnly requires a prepared Git index inventory"
                ))?;
                let mut findings = Vec::new();
                for target_root in &target_roots {
                    if !snapshot.git_index_available_for(target_root) {
                        anyhow::bail!("nextjs-redirect-destinations trackedRoutesOnly requires a prepared Git index inventory for {}", target_root.display());
                    }
                    // A non-Git umbrella's fallback list must never prove
                    // membership in its nested project's Git index.
                    let inventory = snapshot.tracked_paths_for(target_root);
                    let tracked = inventory.iter().filter(|path| {
                        path.starts_with(target_root)
                            && super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                    }).cloned().collect::<Vec<_>>();
                    let tracked = super::path_filter::filter_rule_files(root, config, rule, &tracked)?;
                    findings.extend(scan(root, &opts, &files, &tracked, std::slice::from_ref(target_root), sources));
                }
                findings
            } else {
                scan(root, &opts, &files, &files, &target_roots, sources)
            };
            Ok(findings)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
