use super::{
    step::{collect_step_filters_with_sources, StepContext},
    workflow_finding, workflow_path_filters, CiFilter, RuleFinding, WorkflowSelector,
};
use serde_yaml::Value;

pub(super) fn from_workflow(
    root: &std::path::Path,
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
            )
        }
    };
    from_value(root, rel, &value, selectors, sources)
}

pub(super) fn from_value(
    root: &std::path::Path,
    rel: &str,
    value: &Value,
    selectors: &[WorkflowSelector],
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    let workflow_paths = workflow_path_filters(value);
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
