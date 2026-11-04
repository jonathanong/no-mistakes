use super::github_actions_script_expressions::{step_script_lines, StepScript};
use super::github_actions_workflow_files::gha_kind;
use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod analyze;
pub const RULE_ID: &str = "github-actions-max-inline-script-lines";

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    max_lines: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self { max_lines: 10 }
    }
}

super::borrowed_source_rule_entrypoints!();

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    super::check_sourced_standalone(&root, config, check_with_files)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = super::run_rule_applications(config, RULE_ID, |rule| {
        let opts: Options = rule.try_rule_options()?;
        anyhow::ensure!(
            opts.max_lines > 0,
            "{RULE_ID} maxLines must be a positive integer"
        );
        let files = super::selected_rule_files(root, config, rule, files)?;
        Ok(files.par_iter().flat_map(|path| {
            let rel = relative_slash_path(root, path);
            let Some(action) = gha_kind(&rel) else { return Vec::new() };
            match sources.read_path(path) {
                Ok(source) => analyze::check_source(&rel, &source, action, opts.max_lines),
                Err(error) => super::github_actions_workflow_files::unreadable_finding(
                    &error, defer_suppression, |line| finding(&rel, line,
                        format!("Cannot read GitHub Actions YAML as UTF-8 ({error}); restore a readable file so inline script length can be checked.")),
                ).into_iter().collect(),
            }
        }).collect())
    })?;
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
