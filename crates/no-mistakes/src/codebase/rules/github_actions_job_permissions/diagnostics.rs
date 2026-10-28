use super::findings::{inherited, unresolved};
use super::{permissions, Options, RULE_ID};
use crate::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let _: Options = rule.try_rule_options()?;
        let selected = rules::selected_rule_files(root, config, rule, files)?;
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let per_file: Vec<_> = selected
            .par_iter()
            .map(|path| check_file(root, path, sources, policy, defer_suppression))
            .collect();
        for result in per_file {
            diagnostics.append(result);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut diagnostics.advisories, sources);
    }
    rules::sort_findings(&mut diagnostics.findings);
    rules::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn check_file(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    policy: Unanalyzable,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    let rel = relative_slash_path(root, path);
    if rules::github_actions_workflow_files::gha_kind(&rel) != Some(false) {
        return AnalysisDiagnostics::default();
    }
    match sources.read_path(path) {
        Ok(source) => check_source(&rel, &source, policy),
        Err(error) => {
            let mut diagnostics = AnalysisDiagnostics::default();
            let finding = rules::github_actions_workflow_files::unreadable_finding(
                &error,
                defer_suppression,
                |line| {
                    unresolved(
                        &rel,
                        line,
                        "unanalyzable",
                        &format!("cannot read workflow source as UTF-8 ({error})"),
                    )
                },
            );
            if let Some(finding) = finding {
                diagnostics.unanalyzable(policy, finding);
            }
            diagnostics
        }
    }
}

fn check_source(rel: &str, source: &str, policy: Unanalyzable) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let locations = rules::github_actions_workflow_files::job_lines::collect(source);
    let value: Value = match serde_yaml::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.unanalyzable(
                policy,
                unresolved(
                    rel,
                    error.location().map_or(1, |location| location.line()),
                    "unanalyzable",
                    &format!("invalid YAML ({error})"),
                ),
            );
            return diagnostics;
        }
    };
    let Some(root) = value.as_mapping() else {
        diagnostics.unanalyzable(
            policy,
            unresolved(rel, 1, "unanalyzable", "workflow is not a mapping"),
        );
        return diagnostics;
    };
    let root_permission = root.get(key("permissions"));
    let summary = root_permission
        .map(permissions::inspect)
        .unwrap_or_default();
    if summary.unresolved || root.contains_key(key("<<")) {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.permissions_line(),
                "unanalyzable",
                "workflow permissions are not fully static",
            ),
        );
    }
    let Some(jobs_value) = root.get(key("jobs")) else {
        return diagnostics;
    };
    let Some(jobs) = jobs_value.as_mapping() else {
        diagnostics.unanalyzable(
            policy,
            unresolved(
                rel,
                locations.jobs_line(),
                "unanalyzable",
                "jobs is not a mapping",
            ),
        );
        return diagnostics;
    };
    for (job_key, job_value) in jobs {
        let Some(job_id) = job_key.as_str() else {
            diagnostics.unanalyzable(
                policy,
                unresolved(
                    rel,
                    locations.jobs_line(),
                    "unanalyzable",
                    "job name is not a string",
                ),
            );
            continue;
        };
        let line = locations.line(job_id);
        let Some(job) = job_value.as_mapping() else {
            diagnostics.unanalyzable(
                policy,
                unresolved(rel, line, job_id, "job definition is not a mapping"),
            );
            continue;
        };
        if let Some(own) = job.get(key("permissions")) {
            if permissions::inspect(own).unresolved {
                diagnostics.unanalyzable(
                    policy,
                    unresolved(rel, line, job_id, "job permissions are not fully static"),
                );
            }
            continue;
        }
        if job.contains_key(key("<<")) {
            diagnostics.unanalyzable(
                policy,
                unresolved(rel, line, job_id, "job merge may supply permissions"),
            );
        }
        if !summary.writes.is_empty() {
            diagnostics
                .findings
                .push(inherited(rel, line, job_id, &summary.writes));
        }
    }
    diagnostics
}

fn key(name: &str) -> Value {
    Value::String(name.to_owned())
}
