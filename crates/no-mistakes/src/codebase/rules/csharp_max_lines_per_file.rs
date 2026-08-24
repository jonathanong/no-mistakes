use crate::codebase::ts_source::{discover_with_extensions, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod prepared;
mod scan;

use prepared::{is_csharp_file, PreparedRule};
use scan::scan;

use super::RuleFinding;

pub const RULE_ID: &str = "csharp-max-lines-per-file";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.try_rule_options()?;
        let prepared = PreparedRule::new(&opts, root, &super::target_roots(root, config, rule))?;
        let files: Vec<PathBuf> = prepared
            .roots
            .iter()
            .flat_map(|rule_root| discover_with_extensions(rule_root, skip, &["cs"]))
            .filter(|path| !prepared.is_excluded(root, path))
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &prepared, &files, None, false)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
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
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let skip = super::skip_dir_set(config);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.try_rule_options()?;
        let prepared = PreparedRule::new(&opts, root, &super::target_roots(root, config, rule))?;
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                is_csharp_file(path)
                    && super::file_allowed_by_roots_and_skip(root, &skip, path, &prepared.roots)
                    && !prepared.is_excluded(root, path)
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(
            root,
            &prepared,
            &files,
            Some(sources),
            defer_suppression,
        )?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
