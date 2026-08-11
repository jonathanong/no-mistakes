use serde_yaml::{Mapping, Value};

mod activity;
mod cron;
mod dispatch;
mod shape;

use activity::activity_type_config_valid;
use cron::schedule_config_valid;
use dispatch::workflow_dispatch_config_valid;
pub(super) use shape::{has_workflow_call_trigger, workflow_call_trigger_keys_valid};

pub(super) fn workflow_trigger_configs_valid(on: &Value) -> bool {
    match on {
        Value::Mapping(triggers) => triggers.iter().all(|(trigger, config)| {
            trigger
                .as_str()
                .is_some_and(|trigger| trigger_config_valid(trigger, config))
        }),
        Value::String(trigger) => trigger_config_not_required(trigger),
        Value::Sequence(triggers) => triggers
            .iter()
            .all(|trigger| trigger.as_str().is_some_and(trigger_config_not_required)),
        _ => false,
    }
}

fn trigger_config_not_required(trigger: &str) -> bool {
    !matches!(trigger, "schedule" | "workflow_run")
}

/// Only credit workflow files whose trigger configuration can be represented
/// faithfully by the trigger evaluator. GitHub rejects unknown event options,
/// so accepting them here would credit an unschedulable typecheck gate.
fn trigger_config_valid(trigger: &str, config: &Value) -> bool {
    if matches!(config, Value::Null) {
        return trigger_config_not_required(trigger);
    }
    match trigger {
        "schedule" => schedule_config_valid(config),
        "workflow_call" => config.as_mapping().is_some(),
        "workflow_dispatch" => workflow_dispatch_config_valid(config),
        "workflow_run" => workflow_run_config_valid(config),
        "repository_dispatch" => only_sequence_fields(config, &["types"]),
        "image_version" => only_sequence_fields(config, &["names", "versions"]),
        "push" => ref_filter_config_valid(config, true),
        "pull_request" | "pull_request_target" => pull_request_config_valid(config),
        "merge_group" => merge_group_config_valid(config),
        trigger if activity::ACTIVITY_TYPE_TRIGGERS.contains(&trigger) => {
            activity_type_config_valid(trigger, config)
        }
        _ => config.as_mapping().is_some_and(Mapping::is_empty),
    }
}

fn only_sequence_fields(config: &Value, allowed: &[&str]) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        only_keys(mapping, allowed) && mapping.values().all(non_empty_literal_string_sequence)
    })
}

fn ref_filter_config_valid(config: &Value, allow_tags: bool) -> bool {
    let allowed = if allow_tags {
        &[
            "branches",
            "branches-ignore",
            "tags",
            "tags-ignore",
            "paths",
            "paths-ignore",
        ][..]
    } else {
        &["branches", "branches-ignore", "paths", "paths-ignore"][..]
    };
    config.as_mapping().is_some_and(|mapping| {
        only_keys(mapping, allowed)
            && mapping.values().all(non_empty_literal_string_sequence)
            && negated_patterns_have_positive(mapping)
            && mutually_exclusive(mapping, "branches", "branches-ignore")
            && mutually_exclusive(mapping, "paths", "paths-ignore")
            && (!allow_tags || mutually_exclusive(mapping, "tags", "tags-ignore"))
    })
}

fn pull_request_config_valid(config: &Value) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        only_keys(
            mapping,
            &[
                "types",
                "branches",
                "branches-ignore",
                "paths",
                "paths-ignore",
            ],
        ) && mapping.iter().all(|(key, value)| {
            key.as_str() != Some("types")
                || activity::string_sequence_values_valid(
                    value,
                    activity::PULL_REQUEST_ACTIVITY_TYPES,
                )
        }) && mapping.iter().all(|(key, value)| {
            key.as_str() == Some("types") || non_empty_literal_string_sequence(value)
        }) && negated_patterns_have_positive(mapping)
            && mutually_exclusive(mapping, "branches", "branches-ignore")
            && mutually_exclusive(mapping, "paths", "paths-ignore")
    })
}

fn merge_group_config_valid(config: &Value) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        only_keys(mapping, &["types", "branches", "branches-ignore"])
            && mapping.iter().all(|(key, value)| {
                key.as_str() != Some("types")
                    || activity::string_sequence_values_valid(value, &["checks_requested"])
            })
            && mapping.iter().all(|(key, value)| {
                key.as_str() == Some("types") || non_empty_literal_string_sequence(value)
            })
            && negated_patterns_have_positive(mapping)
            && mutually_exclusive(mapping, "branches", "branches-ignore")
    })
}

fn workflow_run_config_valid(config: &Value) -> bool {
    config.as_mapping().is_some_and(|mapping| {
        only_keys(
            mapping,
            &["workflows", "types", "branches", "branches-ignore"],
        ) && mapping.iter().all(|(key, value)| {
            key.as_str() != Some("types")
                || activity::string_sequence_values_valid(
                    value,
                    &["completed", "requested", "in_progress"],
                )
        }) && mapping.iter().all(|(key, value)| {
            key.as_str() == Some("types") || non_empty_literal_string_sequence(value)
        }) && negated_patterns_have_positive(mapping)
            && mutually_exclusive(mapping, "branches", "branches-ignore")
            && mapping
                .get("workflows")
                .is_some_and(non_empty_literal_string_sequence)
    })
}

pub(super) fn non_empty_literal_string_sequence(value: &Value) -> bool {
    value.as_sequence().is_some_and(|values| {
        !values.is_empty()
            && values.iter().all(|value| {
                value.as_str().is_some_and(|value| {
                    !value.trim().is_empty() && !value.contains("${{") && !value.contains("}}")
                })
            })
    })
}

fn mutually_exclusive(mapping: &Mapping, first: &str, second: &str) -> bool {
    !(mapping.contains_key(first) && mapping.contains_key(second))
}

fn negated_patterns_have_positive(mapping: &Mapping) -> bool {
    ["branches", "tags", "paths"].iter().all(|key| {
        mapping.get(*key).is_none_or(|value| {
            let patterns = value.as_sequence().expect("filter shape was validated");
            !patterns.iter().any(|pattern| {
                pattern
                    .as_str()
                    .is_some_and(|pattern| pattern.starts_with('!'))
            }) || patterns.iter().any(|pattern| {
                pattern
                    .as_str()
                    .is_some_and(|pattern| !pattern.starts_with('!'))
            })
        })
    })
}

fn only_keys(mapping: &Mapping, allowed: &[&str]) -> bool {
    mapping
        .keys()
        .all(|key| key.as_str().is_some_and(|key| allowed.contains(&key)))
}
