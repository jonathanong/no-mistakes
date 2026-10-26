use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod context;
mod field;
mod globs;
mod limits;
pub(crate) mod paths;

pub const RULE_ID: &str = "no-mistakes-config";

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
    if !config.rule_configured(RULE_ID) {
        return Ok(Vec::new());
    }
    let tracked = tracked_rels(root, all_files);
    let context = context::Context::new(root, config, all_files, sources);
    let mut findings = paths::lint(config, &tracked, &context)?;
    findings.extend(globs::lint(config, &tracked, &context)?);
    findings.extend(limits::lint(config, &context));
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn tracked_rels(root: &Path, all_files: &[PathBuf]) -> BTreeSet<String> {
    all_files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .filter(|rel| !rel.is_empty())
        .collect()
}

pub(crate) fn config_rel(root: &Path, all_files: &[PathBuf]) -> String {
    all_files
        .iter()
        .find_map(|path| {
            let name = path.file_name()?.to_str()?;
            (name == ".no-mistakes.yml" || name == ".no-mistakes.yaml")
                .then(|| relative_slash_path(root, path))
        })
        .unwrap_or_else(|| ".no-mistakes.yml".to_string())
}

#[cfg(test)]
mod location_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod selector_tests;
