mod values;

use super::{
    globs::{compile_patterns, PredicateQuantifier},
    RuleFinding, RULE_ID,
};
use crate::codebase::ci_graph::{discover_workflow_files, relative_slash};
use crate::config::v2::schema::NoMistakesConfig;
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;
use values::{filter_patterns, parse_filters_value};

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
    pub(super) compiled: Vec<super::globs::CompiledGlob>,
    pub(super) quantifier: PredicateQuantifier,
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
                rel,
                job_id,
                step_id,
                step,
                &mut filters,
                &mut findings,
            );
        }
    }
    (filters, findings)
}

fn collect_step_filters(
    root: &Path,
    rel: &str,
    job_id: &str,
    step_id: &str,
    step: &Value,
    filters: &mut Vec<CiFilter>,
    findings: &mut Vec<RuleFinding>,
) {
    if !is_paths_filter_step(step) {
        return;
    }
    let Some(with) = step.get("with") else { return };
    let Some(raw_filters) = with.get("filters").and_then(Value::as_str) else {
        return;
    };
    let quantifier = predicate_quantifier(with);
    let Some(parsed) = parse_filters_value(root, rel, job_id, step_id, raw_filters, findings)
    else {
        return;
    };
    let Some(map) = parsed.as_mapping() else {
        return;
    };
    for (name, patterns) in map {
        let Some(name) = name.as_str() else { continue };
        let patterns = filter_patterns(patterns);
        let compiled = match compile_patterns(&patterns) {
            Ok(compiled) => compiled,
            Err(error) => {
                findings.push(workflow_finding(
                    rel,
                    format!("{rel}: filter `{name}` contains invalid glob: {error}"),
                    Some(name.to_string()),
                ));
                continue;
            }
        };
        filters.push(CiFilter {
            workflow: rel.to_string(),
            name: name.to_string(),
            compiled,
            quantifier,
        });
    }
}

fn is_paths_filter_step(step: &Value) -> bool {
    step.get("uses")
        .and_then(Value::as_str)
        .is_some_and(|uses| uses.trim().starts_with("dorny/paths-filter"))
}

fn predicate_quantifier(with: &Value) -> PredicateQuantifier {
    match with
        .get("predicate-quantifier")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "every" => PredicateQuantifier::Every,
        _ => PredicateQuantifier::Some,
    }
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
