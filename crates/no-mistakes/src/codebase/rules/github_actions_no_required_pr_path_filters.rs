use super::file_diagnostics::{self, InputUncertainty};
use super::github_actions_workflow_files::gha_kind;
use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{ensure, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "github_actions_no_required_pr_path_filters/inspect.rs"]
mod inspect;

pub const RULE_ID: &str = "github-actions-no-required-pr-path-filters";

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    workflows: BTreeSet<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

super::diagnostic_source_rule_entrypoints!();

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        for workflow in &options.workflows {
            ensure!(
                gha_kind(workflow) == Some(false) && !workflow.contains(['\\', '*', '?', '[', ']']),
                "{RULE_ID}: workflows entries must be literal paths directly under .github/workflows (got {workflow:?})"
            );
        }
        if options.workflows.is_empty() {
            continue;
        }
        let selected = super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |path| options.workflows.contains(&relative_slash_path(root, path)),
        )?;
        let selected_paths: BTreeSet<String> = selected
            .iter()
            .map(|path| relative_slash_path(root, path))
            .collect();
        let missing: Vec<&str> = options
            .workflows
            .iter()
            .filter(|workflow| !selected_paths.contains(workflow.as_str()))
            .map(String::as_str)
            .collect();
        ensure!(
            missing.is_empty(),
            "{RULE_ID}: configured workflows matched no workflow file: {}",
            missing.join(", ")
        );
        let findings = selected
            .par_iter()
            .flat_map(|path| {
                let rel = relative_slash_path(root, path);
                inspect::check_file(&rel, path, sources)
            })
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
