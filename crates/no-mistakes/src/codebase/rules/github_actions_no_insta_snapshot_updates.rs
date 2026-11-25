use super::analysis_policy::AnalysisDiagnostics;
use super::file_diagnostics::{self, InputUncertainty};
use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod run;
mod scan;

pub const RULE_ID: &str = "github-actions-no-insta-snapshot-updates";
const NAME: &str = "INSTA_UPDATE";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let _opts: Options = rule.try_rule_options()?;
        let files = super::selected_rule_files(root, config, rule, all_files)?;
        let findings: Vec<RuleFinding> = files
            .par_iter()
            .flat_map(|path| scan::check_workflow(root, path, sources))
            .collect();
        output.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(output))
}

#[cfg(test)]
mod tests;
