mod step;
mod values;
mod workflow_paths;

use super::{
    globs::{selected_by, PredicateQuantifier},
    RuleFinding, RULE_ID,
};
use crate::codebase::ci_graph::{discover_workflow_files, relative_slash};
use crate::config::v2::schema::NoMistakesConfig;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;
use step::{collect_step_filters, StepContext};
use workflow_paths::workflow_path_filters;

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
    workflow_paths: Vec<Vec<super::globs::CompiledGlob>>,
}

impl CiFilter {
    pub(super) fn workflow_allows(&self, path: &str) -> bool {
        self.workflow_paths.is_empty()
            || self
                .workflow_paths
                .iter()
                .any(|patterns| selected_by(patterns, path))
    }
}

pub(super) fn ci_filters(
    root: &Path,
    config: &NoMistakesConfig,
    selectors: &[WorkflowSelector],
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    for path in discover_workflow_files(root, &config.ci) {
        let rel = relative_slash(root, &path);
        if !selectors.is_empty()
            && !selectors
                .iter()
                .any(|selector| selector.path.is_empty() || selector.path == rel)
        {
            continue;
        }
        let source = match std::fs::read_to_string(&path) {
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
            extract_filters_from_workflow(root, &rel, &source, selectors);
        filters.extend(workflow_filters);
        findings.extend(workflow_findings);
    }
    filters.sort_by(|a, b| (&a.workflow, &a.name).cmp(&(&b.workflow, &b.name)));
    (filters, findings)
}

fn extract_filters_from_workflow(
    root: &Path,
    rel: &str,
    source: &str,
    selectors: &[WorkflowSelector],
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
            collect_step_filters(
                root,
                StepContext {
                    rel,
                    job_id,
                    step_id,
                    workflow_paths: &workflow_paths,
                },
                step,
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
