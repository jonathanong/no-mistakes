use super::RuleFinding;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod analyze;
mod expressions;
mod hit;
mod line_scan;
mod value_walk;
mod yaml_parse;
mod yaml_text;

pub(crate) use line_scan::{step_run_lines, step_uses_lines, StepRun, StepUse};

pub const RULE_ID: &str = "github-actions-script-expressions";

/// Full-match patterns for constant contexts. `None` for `constantContexts`
/// selects these; an explicit list, including `[]`, replaces them.
pub(crate) const DEFAULT_CONSTANT_CONTEXTS: &[&str] = &[
    r"^github\.(repository|repository_owner|server_url|run_id|run_number|run_attempt|sha|event_name|workspace)$",
    r"^runner\.(os|arch|temp)$",
    r#"^job\.services\.[A-Za-z0-9_-]+\.ports\[(?:'[0-9]+'|"[0-9]+")\]$"#,
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    /// `None` (option omitted) uses [`DEFAULT_CONSTANT_CONTEXTS`]. `Some` replaces
    /// that list, so `[]` flags every expression.
    pub(crate) constant_contexts: Option<Vec<String>>,
}

super::borrowed_source_rule_entrypoints!();

pub fn check(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, defer_suppression)
        .map(|diagnostics| diagnostics.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    diagnostics::run(root, config, files, sources, defer_suppression)
}

mod diagnostics;

fn compile_patterns(opts: &Options) -> Result<Vec<Regex>> {
    let configured = opts
        .constant_contexts
        .clone()
        .unwrap_or_else(default_constant_contexts);
    configured
        .into_iter()
        .map(|pattern| {
            Regex::new(&pattern).map_err(|err| {
                anyhow::anyhow!(
                    "{RULE_ID} constantContexts pattern `{pattern}` is not a valid regex: {err}"
                )
            })
        })
        .collect()
}

pub(crate) fn default_constant_contexts() -> Vec<String> {
    DEFAULT_CONSTANT_CONTEXTS
        .iter()
        .map(|pattern| (*pattern).to_string())
        .collect()
}

#[cfg(test)]
mod tests;
