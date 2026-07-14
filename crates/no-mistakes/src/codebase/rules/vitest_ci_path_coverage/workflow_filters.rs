mod step;
mod values;
mod workflow_paths;

use super::{
    globs::{selected_by, PredicateQuantifier},
    RuleFinding, RULE_ID,
};
use crate::codebase::ci_graph::{discover_workflow_files_from_snapshot, relative_slash};
use crate::config::v2::schema::NoMistakesConfig;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;
use step::{collect_step_filters_with_sources, StepContext};
use workflow_paths::{workflow_path_filters, WorkflowPathFilters};

#[cfg(test)]
mod tests;

#[derive(Clone, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct WorkflowSelector {
    pub(crate) path: String,
    pub(crate) job: String,
    pub(crate) step_id: String,
}

#[derive(Debug)]
pub(super) struct CiFilter {
    pub(super) workflow: String,
    pub(super) name: String,
    pub(super) compiled: Vec<Vec<super::globs::CompiledGlob>>,
    pub(super) quantifier: PredicateQuantifier,
    workflow_paths: WorkflowPathFilters,
}

impl CiFilter {
    pub(super) fn workflow_allows(&self, path: &str) -> bool {
        self.workflow_paths.allows(path)
    }
}

pub(super) fn ci_filters_from_snapshot_with_sources(
    root: &Path,
    config: &NoMistakesConfig,
    selectors: &[WorkflowSelector],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    ci_filters_from_paths(
        root,
        selectors,
        discover_workflow_files_from_snapshot(root, &config.ci, snapshot),
        sources,
    )
}

fn ci_filters_from_paths(
    root: &Path,
    selectors: &[WorkflowSelector],
    workflow_files: Vec<std::path::PathBuf>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    for path in workflow_files {
        let rel = relative_slash(root, &path);
        if !selectors.is_empty()
            && !selectors
                .iter()
                .any(|selector| selector.path.is_empty() || selector.path == rel)
        {
            continue;
        }
        let source = match sources.read_path(&path) {
            Ok(source) => source,
            Err(error) => {
                findings.push(workflow_finding(
                    &rel,
                    format!("{rel}: could not read workflow file: {error}"),
                    None,
                ));
                continue;
            }
        };
        let (workflow_filters, workflow_findings) =
            extract_filters_from_workflow_with_sources(root, &rel, &source, selectors, sources);
        filters.extend(workflow_filters);
        findings.extend(workflow_findings);
    }
    filters.sort_by(|a, b| (&a.workflow, &a.name).cmp(&(&b.workflow, &b.name)));
    (filters, findings)
}

fn extract_filters_from_workflow_with_sources(
    root: &Path,
    rel: &str,
    source: &str,
    selectors: &[WorkflowSelector],
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let value: Value = match serde_yaml::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            return (
                Vec::new(),
                vec![workflow_finding(
                    rel,
                    format!("{rel}: could not parse workflow YAML: {error}"),
                    None,
                )],
            );
        }
    };
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    let workflow_paths = workflow_path_filters(&value);
    let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) else {
        return (filters, findings);
    };
    for (job_key, job) in jobs {
        let job_id = job_key.as_str().unwrap_or_default();
        let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
            continue;
        };
        for step in steps {
            let step_id = step.get("id").and_then(Value::as_str).unwrap_or_default();
            if !selectors.is_empty()
                && !selectors.iter().any(|selector| {
                    (selector.path.is_empty() || selector.path == rel)
                        && (selector.job.is_empty() || selector.job == job_id)
                        && (selector.step_id.is_empty() || selector.step_id == step_id)
                })
            {
                continue;
            }
            collect_step_filters_with_sources(
                root,
                StepContext {
                    rel,
                    job_id,
                    step_id,
                    workflow_paths: &workflow_paths,
                },
                step,
                sources,
                &mut filters,
                &mut findings,
            );
        }
    }
    (filters, findings)
}

pub(super) fn workflow_finding(file: &str, message: String, target: Option<String>) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target,
    }
}
