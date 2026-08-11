use super::conditions::direct_inputs;
use super::ParsedWorkflowSet;
use crate::codebase::ci_graph::{parse::parse_workflow_value, triggers::CompiledTriggers};
use crate::codebase::workflow_topology::workflow_values;
use std::collections::BTreeSet;

use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;

mod activation;
mod events;
#[cfg(test)]
mod matrix_property_tests;
#[cfg(test)]
mod memo_tests;
pub(crate) mod model;
mod steps;
mod validation;

use activation::scan_activation;
use events::source_change_event_contexts;
use model::{ActivationMemo, ActivationState, ScanContext, WorkflowDocument};
use validation::{workflow_call_shape_valid, workflow_shape_valid};

pub(super) fn collect_ci_projects_with_stats(
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    project_source_inputs: &ProjectSourceInputs,
) -> (BTreeSet<String>, usize) {
    let workflows = parsed
        .documents
        .iter()
        .filter_map(|document| {
            let value = document.value.as_ref().ok()?;
            if !workflow_shape_valid(value) {
                return None;
            }
            Some((
                document.path.clone(),
                WorkflowDocument {
                    value,
                    call_contract: workflow_values::parse_workflow_call(value.get("on")),
                    call_contract_shape_valid: workflow_call_shape_valid(value.get("on")),
                },
            ))
        })
        .collect();
    let context = ScanContext {
        workflows,
        tracked,
        project_source_inputs,
    };
    let mut projects = BTreeSet::new();
    let mut computations = 0;
    for (path, document) in &context.workflows {
        if !document.call_contract_shape_valid {
            continue;
        }
        let trigger_model = parse_workflow_value(document.value, path);
        if trigger_model.triggers.events.is_empty() {
            continue;
        }
        for event_name in trigger_model.triggers.events.keys() {
            // A direct workflow's call graph may be the same for multiple events,
            // but its path-filtered coverage is event-specific.
            let mut memo = ActivationMemo::new();
            let triggers = CompiledTriggers::for_event(&trigger_model, event_name)
                .expect("event came from the trigger model");
            for event in source_change_event_contexts(document.value, event_name) {
                let Some(inputs) = direct_inputs(document.call_contract.as_ref(), &event) else {
                    continue;
                };
                if let Some(activation_projects) = scan_activation(
                    path,
                    document,
                    &triggers,
                    &ActivationState::direct(inputs),
                    &context,
                    &mut memo,
                ) {
                    projects.extend(activation_projects);
                }
            }
            computations += memo.computations();
        }
    }
    (projects, computations)
}

#[cfg(test)]
mod activity_tests;
#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod secret_tests;
#[cfg(test)]
mod tests;
