//! Hand-walks parsed workflow YAML into typed model fragments, ported from
//! `workflow-values.mts`. Reused by [`super::parse`] for every workflow
//! file and job entry.
//!
//! Artifact declaration parsing (`step.with` → an `ArtifactDeclaration`) is
//! intentionally not ported yet — [`parse_steps`] always leaves
//! `WorkflowStep::artifact` as `None`. A later port wave adds it alongside
//! the artifact-dataflow resolver.

use super::model;
use super::value_primitives;
use serde_yaml::Value;
use std::collections::BTreeMap;

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

pub fn parse_steps(value: Option<&Value>) -> Vec<model::WorkflowStep> {
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
            Some(model::WorkflowStep {
                index: position as u32,
                kind,
                id: value_primitives::string_value(step.get("id")),
                name: value_primitives::string_value(step.get("name")),
                condition: value_primitives::string_value(step.get("if")),
                uses,
                artifact: None,
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

pub fn parse_workflow_call(value: Option<&Value>) -> Option<model::WorkflowCallContract> {
    let is_callable = match value {
        Some(Value::String(text)) => text == "workflow_call",
        Some(Value::Sequence(items)) => items
            .iter()
            .any(|item| item.as_str() == Some("workflow_call")),
        Some(v) if value_primitives::is_record(Some(v)) => v.get("workflow_call").is_some(),
        _ => false,
    };
    if !is_callable {
        return None;
    }
    // The string/array forms set `config = null` in the TS engine; only the
    // mapping form (`on: { workflow_call: {...} }`) carries a real config.
    let config = match value {
        Some(v) if value_primitives::is_record(Some(v)) => v.get("workflow_call"),
        _ => None,
    };
    let mapping = config.filter(|c| value_primitives::is_record(Some(c)));
    Some(model::WorkflowCallContract {
        inputs: parse_declarations(
            mapping.and_then(|m| m.get("inputs")),
            parse_workflow_call_input,
        ),
        secrets: parse_declarations(
            mapping.and_then(|m| m.get("secrets")),
            parse_workflow_call_secret,
        ),
        outputs: parse_declarations(
            mapping.and_then(|m| m.get("outputs")),
            parse_workflow_call_output,
        ),
    })
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

fn parse_declarations<T>(
    value: Option<&Value>,
    parse: impl Fn(Option<&Value>) -> T,
) -> BTreeMap<String, T> {
    let Some(Value::Mapping(mapping)) = value else {
        return BTreeMap::new();
    };
    mapping
        .iter()
        .filter_map(|(key, declaration)| Some((key_name(key)?, parse(Some(declaration)))))
        .collect()
}

fn parse_workflow_call_input(value: Option<&Value>) -> model::WorkflowCallInput {
    let input_type = value
        .and_then(|v| v.get("type"))
        .and_then(Value::as_str)
        .and_then(|text| match text {
            "boolean" => Some(model::WorkflowCallInputType::Boolean),
            "number" => Some(model::WorkflowCallInputType::Number),
            "string" => Some(model::WorkflowCallInputType::String),
            _ => None,
        });
    let required = value
        .and_then(|v| v.get("required"))
        .and_then(Value::as_bool)
        == Some(true);
    let default = value.and_then(|v| v.get("default")).and_then(scalar_value);
    let description = value
        .and_then(|v| v.get("description"))
        .and_then(Value::as_str)
        .map(str::to_string);
    model::WorkflowCallInput {
        input_type,
        required,
        default,
        description,
    }
}

fn parse_workflow_call_secret(value: Option<&Value>) -> model::WorkflowCallSecret {
    model::WorkflowCallSecret {
        required: value
            .and_then(|v| v.get("required"))
            .and_then(Value::as_bool)
            == Some(true),
        description: value
            .and_then(|v| v.get("description"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn parse_workflow_call_output(value: Option<&Value>) -> model::WorkflowCallOutput {
    model::WorkflowCallOutput {
        value: value
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
            .map(str::to_string),
        description: value
            .and_then(|v| v.get("description"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn scalar_record(value: Option<&Value>) -> BTreeMap<String, model::JsonScalar> {
    let Some(Value::Mapping(mapping)) = value else {
        return BTreeMap::new();
    };
    mapping
        .iter()
        .filter_map(|(key, item)| Some((key_name(key)?, scalar_value(item)?)))
        .collect()
}

fn scalar_value(value: &Value) -> Option<model::JsonScalar> {
    match value {
        Value::String(text) => Some(model::JsonScalar::Text(text.clone())),
        Value::Bool(flag) => Some(model::JsonScalar::Bool(*flag)),
        Value::Number(number) => {
            value_primitives::yaml_number_to_json(number).map(model::JsonScalar::Number)
        }
        _ => None,
    }
}
