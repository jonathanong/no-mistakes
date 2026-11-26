use super::{malformed_workflow, model, parse_expanded, ParsedWorkflow};
use crate::codebase::structured_value::{PartialYaml, StructuredValueError};
use serde_yaml::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowMergeUncertainty {
    pub(crate) job_inventory: bool,
    pub(crate) concurrency: bool,
    pub(crate) callable: bool,
    pub(crate) call_contract: bool,
    pub(crate) jobs: BTreeMap<String, JobMergeUncertainty>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JobMergeUncertainty {
    pub(crate) needs: bool,
    pub(crate) steps: bool,
    pub(crate) artifacts: bool,
    pub(crate) concurrency: bool,
    pub(crate) uses: bool,
}

impl WorkflowMergeUncertainty {
    pub(super) fn from_yaml(partial: &PartialYaml) -> Self {
        let workflow = partial.value();
        let jobs = workflow.get("jobs");
        let mut uncertain = Self {
            job_inventory: jobs.map_or_else(
                || partial.mapping_unknown(workflow),
                |jobs| partial.mapping_unknown(jobs),
            ),
            concurrency: field_unknown(partial, workflow, "concurrency"),
            callable: workflow.get("on").map_or_else(
                || partial.mapping_unknown(workflow),
                |on| on.get("workflow_call").is_none() && partial.mapping_unknown(on),
            ),
            call_contract: workflow
                .get("on")
                .and_then(|on| on.get("workflow_call"))
                .is_some_and(|contract| partial.contains_unknown(contract)),
            jobs: BTreeMap::new(),
        };
        if let Some(jobs) = jobs.and_then(Value::as_mapping) {
            for (key, job) in jobs {
                if let Some(key) = super::workflow_values::key_name(key) {
                    let scope = JobMergeUncertainty {
                        needs: field_unknown(partial, job, "needs"),
                        steps: match job.get("steps") {
                            Some(Value::Sequence(steps)) => {
                                steps.iter().any(|step| partial.mapping_unknown(step))
                            }
                            Some(_) => false,
                            None => partial.mapping_unknown(job),
                        },
                        artifacts: field_unknown(partial, job, "steps")
                            || field_unknown(partial, job, "strategy"),
                        concurrency: field_unknown(partial, job, "concurrency"),
                        uses: field_unknown(partial, job, "uses"),
                    };
                    if scope.needs
                        || scope.steps
                        || scope.artifacts
                        || scope.concurrency
                        || scope.uses
                    {
                        uncertain.jobs.insert(key, scope);
                    }
                }
            }
        }
        uncertain
    }
}

fn field_unknown(partial: &PartialYaml, owner: &Value, key: &str) -> bool {
    owner.get(key).map_or_else(
        || partial.mapping_unknown(owner),
        |field| partial.contains_unknown(field),
    )
}

pub(super) fn parse(path: &str, error: StructuredValueError) -> ParsedWorkflow {
    let Some(partial) = error.partial else {
        return malformed_workflow(path, &error.message);
    };
    let uncertainty = WorkflowMergeUncertainty::from_yaml(&partial);
    let mut workflow = parse_expanded(partial.value(), path, uncertainty.job_inventory);
    if uncertainty.call_contract {
        workflow.node.workflow_call = None;
    }
    workflow.diagnostics.push(
        model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::UnlocatedYamlMerge,
            error.message,
            path,
        )
        .with_merge_uncertainty(uncertainty),
    );
    workflow
}
