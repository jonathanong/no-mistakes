use super::conditions::{direct_inputs, with_workflow_name, SecretState};
use super::ParsedWorkflowSet;
use crate::codebase::ci_graph::{parse::parse_workflow_value, triggers::CompiledTriggers};
use crate::codebase::workflow_topology::workflow_values;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::local_actions::LocalActionCatalog;
use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;

mod activation;
mod events;
#[cfg(test)]
mod matrix_property_tests;
#[cfg(test)]
mod memo_tests;
pub(crate) mod model;
mod steps;
pub(crate) mod validation;

use activation::scan_activation;
use events::source_change_event_contexts;
use model::{ActivationMemo, ActivationState, ScanContext, WorkflowDocument};
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) use validation::valid_static_container_image_reference;
use validation::{workflow_call_shape_valid, workflow_shape_valid};

pub(super) fn collect_ci_projects_with_local_actions(
    root: &std::path::Path,
    parsed: &ParsedWorkflowSet,
    tracked: &BTreeSet<String>,
    tracked_paths: &[PathBuf],
    project_source_inputs: &ProjectSourceInputs,
    local_actions: &LocalActionCatalog,
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
        visible_paths: tracked_paths
            .iter()
            .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
            .collect(),
        project_source_inputs,
        local_actions,
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
            let mut event_projects: Option<BTreeSet<String>> = None;
            for event in source_change_event_contexts(document.value, event_name) {
                let projects = direct_inputs(document.call_contract.as_ref(), &event)
                    .map(|inputs| with_workflow_name(inputs, document.value, path))
                    .and_then(|inputs| {
                        // A direct pull request can originate from a fork.
                        // Without origin facts, never credit a secret-dependent path.
                        let secrets = if event.name == "pull_request" {
                            SecretState::unavailable()
                        } else {
                            SecretState::direct()
                        };
                        scan_activation(
                            path,
                            document,
                            &triggers,
                            &ActivationState::direct(inputs, secrets),
                            &context,
                            &mut memo,
                        )
                    })
                    .map_or_else(BTreeSet::new, |activation| activation.projects);
                // A project is covered only if every exact source-change
                // branch activation can run it, never merely one branch.
                event_projects = Some(match event_projects {
                    Some(covered) => covered.intersection(&projects).cloned().collect(),
                    None => projects,
                });
            }
            if !memo.exhausted() {
                projects.extend(event_projects.unwrap_or_default());
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
mod test_support;
#[cfg(test)]
mod tests;
