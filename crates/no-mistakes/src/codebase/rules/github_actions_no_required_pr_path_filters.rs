use super::file_diagnostics::{self, InputUncertainty};
use super::github_actions_workflow_files::{gha_kind, job_lines::document::Document};
use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{ensure, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
        let findings = selected
            .par_iter()
            .flat_map(|path| {
                let rel = relative_slash_path(root, path);
                check_file(&rel, path, sources)
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

fn check_file(rel: &str, path: &Path, sources: &SourceStore) -> Vec<RuleFinding> {
    let value = match sources.parse_yaml_path(path) {
        Ok(value) => value,
        Err(error) => {
            return vec![finding(rel, 1, "unanalyzable", format!(
                "Cannot inspect PR trigger filters for {rel} ({error}). Repair the workflow YAML and run the check again."
            ))];
        }
    };
    let Some(triggers) = value.get("on") else {
        return Vec::new();
    };
    let filters: Vec<_> = ["pull_request", "pull_request_target"]
        .into_iter()
        .flat_map(|event| {
            ["paths", "paths-ignore"]
                .into_iter()
                .filter_map(move |key| {
                    triggers.get(event)?.get(key)?;
                    Some((event, key))
                })
        })
        .collect();
    if filters.is_empty() {
        return Vec::new();
    }
    let locations = sources
        .read_path(path)
        .ok()
        .and_then(|source| Document::parse(&source));
    filters.into_iter().map(|(event, key)| {
        let line = locations.as_ref().and_then(|doc| {
            let trigger = doc.field(*doc.roots().first()?, "on")?;
            doc.field_line(doc.field(trigger, event)?, key)
        }).unwrap_or(1);
        let target = format!("on.{event}.{key}");
        finding(rel, line, &target, format!(
            "{rel}:{line}: remove the workflow-level path filter `{target}` from the required PR trigger. Select work inside the workflow and keep the required gate reporting on every PR."
        ))
    }).collect()
}

fn finding(file: &str, line: usize, target: &str, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: Some(target.into()),
        causes: Vec::new(),
    }
}
