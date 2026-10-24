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
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let mut roots = std::collections::BTreeSet::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.try_rule_options()?;
        roots.extend(
            PreparedRule::new(&opts, root, &super::target_roots(root, config, rule))?.roots,
        );
    }
    let mut files: Vec<_> = roots
        .into_iter()
        .flat_map(|root| {
            discover_with_extensions(&root, &config.filesystem.skip_directories, &["cs"])
        })
        .collect();
    files.sort();
    files.dedup();
    check_with_files(root, config, &files)
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
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let skip = super::skip_dir_set(config);
    let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
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
        let findings = scan(root, &prepared, &files, sources, defer_suppression)?;
        diagnostics.append(super::file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            super::file_diagnostics::InputUncertainty::Read,
        )?);
    }
    Ok(super::file_diagnostics::sorted(diagnostics))
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
