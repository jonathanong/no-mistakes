use super::file_diagnostics::{self, InputUncertainty};
use super::github_actions_workflow_files::gha_kind;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use no_mistakes_rule_kernel::finding::RuleFinding;
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
    workflows: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

super::diagnostic_source_rule_entrypoints!();

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    super::validate_options::collect_config_errors(
        root,
        config_path,
        config,
        RULE_ID,
        collect_application_errors,
        errors,
    );
}

pub(super) fn collect_application_errors(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<BTreeSet<String>> {
    let mut app_errors = BTreeSet::new();
    if let Err(error) = super::path_filter::RulePathFilter::new(root, config, rule) {
        app_errors.insert(format!("{error:#}"));
    }
    let options = rule.strict_rule_options::<Options>()?;
    collect_option_errors(&options, &mut app_errors);
    Ok(app_errors)
}

fn collect_option_errors(options: &Options, errors: &mut BTreeSet<String>) {
    if options.workflows.is_empty() {
        errors.insert(super::validate_options::format_invalid_option(
            RULE_ID,
            "workflows",
            &options.workflows,
            "a non-empty list of literal workflow paths directly under .github/workflows",
        ));
        return;
    }
    for workflow in &options.workflows {
        if gha_kind(workflow) != Some(false) || workflow.contains(['\\', '*', '?', '[', ']']) {
            errors.insert(super::validate_options::format_invalid_option(
                RULE_ID,
                "workflows",
                workflow,
                "a literal path directly under .github/workflows",
            ));
        }
    }
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut errors = Vec::new();
    collect_config_errors(root, None, config, &mut errors);
    super::validate_options::finish(errors)?;
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let workflows: BTreeSet<String> = options.workflows.into_iter().collect();
        let selected = super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |path| workflows.contains(&relative_slash_path(root, path)),
        )?;
        let present_paths: BTreeSet<String> = files
            .iter()
            .map(|path| relative_slash_path(root, path))
            .filter(|rel| workflows.contains(rel))
            .collect();
        let selected_paths: BTreeSet<String> = selected
            .iter()
            .map(|path| relative_slash_path(root, path))
            .collect();
        validate_configured_workflows(&workflows, &present_paths, &selected_paths)?;
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

fn validate_configured_workflows(
    workflows: &BTreeSet<String>,
    present: &BTreeSet<String>,
    selected: &BTreeSet<String>,
) -> Result<()> {
    let mut parts = Vec::new();
    let missing: Vec<&str> = workflows
        .iter()
        .filter(|workflow| !present.contains(workflow.as_str()))
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        parts.push(format!(
            "configured workflows matched no workflow file: {}",
            missing.join(", ")
        ));
    }
    let excluded: Vec<&str> = workflows
        .iter()
        .filter(|workflow| {
            present.contains(workflow.as_str()) && !selected.contains(workflow.as_str())
        })
        .map(String::as_str)
        .collect();
    if !excluded.is_empty() {
        parts.push(format!(
            "configured workflows were excluded by config: {}",
            excluded.join(", ")
        ));
    }
    ensure!(parts.is_empty(), "{RULE_ID}: {}", parts.join("; "));
    Ok(())
}

#[cfg(test)]
#[path = "github_actions_no_required_pr_path_filters/tests.rs"]
mod tests;
