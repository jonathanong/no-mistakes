//! Hand-walks parsed workflow YAML into typed model fragments, ported from
//! `workflow-values.mts`. Reused by [`super::parse`] for every workflow
//! file and job entry.

use super::model;
use super::value_primitives;
use call_contract::scalar_record;
use serde_yaml::Value;

mod call_contract;

pub use call_contract::parse_workflow_call;

/// A mapping key coerced to a string the way a JS object property name
/// would be (YAML mapping keys are practically always strings already;
/// this only matters for the rare non-string key). Also used by
/// [`super::parse`] for job-key entries.
pub(super) fn key_name(key: &Value) -> Option<String> {
    key.as_str()
        .map(str::to_string)
        .or_else(|| value_primitives::string_value(Some(key)))
}

pub fn parse_triggers(value: Option<&Value>) -> Vec<model::WorkflowTrigger> {
    match value {
        Some(Value::String(event)) => vec![model::WorkflowTrigger {
            event: event.clone(),
            config: None,
        }],
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(|item| {
                item.as_str().map(|event| model::WorkflowTrigger {
                    event: event.to_string(),
                    config: None,
                })
            })
            .collect(),
        Some(Value::Mapping(mapping)) => {
            let mut triggers: Vec<model::WorkflowTrigger> = mapping
                .iter()
                .filter_map(|(key, config)| {
                    let event = key_name(key)?;
                    let config =
                        (!matches!(config, Value::Null)).then(|| value_primitives::to_json(config));
                    Some(model::WorkflowTrigger { event, config })
                })
                .collect();
            triggers.sort_by(|left, right| left.event.cmp(&right.event));
            triggers
        }
        _ => Vec::new(),
    }
}

pub fn parse_concurrency(value: Option<&Value>) -> Option<model::WorkflowConcurrency> {
    let value = value?;
    if let Value::String(group) = value {
        return Some(model::WorkflowConcurrency {
            raw: model::ConcurrencyRaw {
                group: group.clone(),
                cancel_in_progress: None,
                queue: None,
            },
            effective: model::ConcurrencyEffective {
                group: group.clone(),
                cancel_in_progress: model::ConcurrencyValue::Bool(false),
                queue: "single".to_string(),
            },
        });
    }
    if !value_primitives::is_record(Some(value)) {
        return None;
    }
    let group = value_primitives::string_value(value.get("group"))?;
    let cancel_in_progress = value_primitives::concurrency_value(value.get("cancel-in-progress"));
    let queue = value_primitives::string_value(value.get("queue"));
    Some(model::WorkflowConcurrency {
        raw: model::ConcurrencyRaw {
            group: group.clone(),
            cancel_in_progress: cancel_in_progress.clone(),
            queue: queue.clone(),
        },
        effective: model::ConcurrencyEffective {
            group,
            cancel_in_progress: cancel_in_progress.unwrap_or(model::ConcurrencyValue::Bool(false)),
            queue: queue.unwrap_or_else(|| "single".to_string()),
        },
    })
}

/// `matrix` is the job's already-[`OrderedJson`](value_primitives::OrderedJson)-converted
/// `strategy.matrix` snapshot (see [`matrix_from_job`]) — passed through
/// unchanged to [`super::artifact_values::parse_artifact_declaration`] for
/// matrix-aware artifact name expansion.
pub fn parse_steps(
    value: Option<&Value>,
    matrix: Option<&value_primitives::OrderedJson>,
) -> Vec<model::WorkflowStep> {
    let Some(Value::Sequence(items)) = value else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(position, step)| {
            if !value_primitives::is_record(Some(step)) {
                return None;
            }
            let uses = value_primitives::string_value(step.get("uses"));
            let run = value_primitives::string_value(step.get("run"));
            let kind = if uses.is_some() {
                model::StepKind::Action
            } else if run.is_some() {
                model::StepKind::Run
            } else {
                model::StepKind::Other
            };
            let artifact = uses.as_deref().and_then(|uses| {
                super::artifact_values::parse_artifact_declaration(uses, step.get("with"), matrix)
            });
            Some(model::WorkflowStep {
                index: position as u32,
                kind,
                id: value_primitives::string_value(step.get("id")),
                name: value_primitives::string_value(step.get("name")),
                condition: value_primitives::string_value(step.get("if")),
                uses,
                artifact,
            })
        })
        .collect()
}

pub fn matrix_from_job(job: &Value) -> Option<value_primitives::OrderedJson> {
    let strategy = job.get("strategy")?;
    if !value_primitives::is_record(Some(strategy)) {
        return None;
    }
    let matrix = strategy.get("matrix")?;
    Some(value_primitives::to_json(matrix))
}

/// Builds a `uses:` reusable-workflow call edge. `to` is populated only for
/// local (`./`) calls and is appended after `bindings` at the real
/// construction site — see the field-order note on
/// [`model::WorkflowCallEdge`].
pub fn call_edge(job_id: &str, target: &str, job: &Value) -> model::WorkflowCallEdge {
    let local = target.starts_with("./");
    let secrets_inherit = job.get("secrets").and_then(Value::as_str) == Some("inherit");
    model::WorkflowCallEdge {
        from: job_id.to_string(),
        target: target.to_string(),
        local,
        bindings: model::WorkflowCallBindings {
            inputs: scalar_record(job.get("with")),
            secrets: if secrets_inherit {
                model::WorkflowCallSecretsBinding::Inherit
            } else {
                model::WorkflowCallSecretsBinding::Explicit {
                    values: scalar_record(job.get("secrets")),
                }
            },
        },
        to: local.then(|| normalize_local_call_target(target)),
    }
}

/// `target.replaceAll("\\","/").replace(/^\.\//u, "")` then
/// `posix.normalize`, exactly matching `callEdge`'s local-target
/// normalization. Only the FIRST leading `./` is stripped by that regex
/// (not repeated, unlike the `--workflow` filter normalizer) — remaining
/// `./`/`..` segments anywhere else are resolved by `normalize`.
fn normalize_local_call_target(target: &str) -> String {
    let slashed = target.replace('\\', "/");
    let stripped = slashed.strip_prefix("./").unwrap_or(&slashed);
    super::posix_path::normalize(stripped)
}
