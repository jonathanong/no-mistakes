use super::{InputState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    GithubEventAction, GithubEventContext, GithubRef,
};
use serde_yaml::Value;

pub(in super::super) const EVENT_NAME_KEY: &str = "\0github.event_name";
pub(in super::super) const EVENT_ACTION_KEY: &str = "\0github.event.action";
pub(in super::super) const WORKFLOW_KEY: &str = "\0github.workflow";
pub(in super::super) const PULL_REQUEST_MERGED_KEY: &str = "\0github.event.pull_request.merged";
pub(in super::super) const REF_KEY: &str = "\0github.ref";
pub(in super::super) const REF_NAME_KEY: &str = "\0github.ref_name";
pub(in super::super) const BASE_REF_KEY: &str = "\0github.base_ref";
pub(in super::super) const HEAD_REF_KEY: &str = "\0github.head_ref";
pub(in super::super) const REF_KIND_KEY: &str = "\0github.ref.kind";
pub(in super::super) const REF_SHAPE_KEY: &str = "\0github.ref.shape";
pub(in super::super) const REF_EXCLUSIONS_KEY: &str = "\0github.ref.exclusions";

pub(in super::super) fn event_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_NAME_KEY).cloned()
}
pub(in super::super) fn event_action_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_ACTION_KEY).cloned()
}
pub(in super::super) fn workflow_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(WORKFLOW_KEY).cloned()
}
pub(in super::super) fn pull_request_merged_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(PULL_REQUEST_MERGED_KEY).cloned()
}
pub(in super::super) fn event_ref_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(REF_NAME_KEY).cloned()
}
pub(in super::super) fn event_ref_type_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(REF_KIND_KEY).cloned()
}
pub(in super::super) fn event_base_ref_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(BASE_REF_KEY).cloned()
}
pub(in super::super) fn event_head_ref_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(HEAD_REF_KEY).cloned()
}

pub(super) fn copy_event_inputs(parent: &InputState, inputs: &mut InputState) {
    for key in [
        EVENT_NAME_KEY,
        EVENT_ACTION_KEY,
        WORKFLOW_KEY,
        PULL_REQUEST_MERGED_KEY,
        REF_KEY,
        REF_NAME_KEY,
        BASE_REF_KEY,
        HEAD_REF_KEY,
        REF_KIND_KEY,
        REF_SHAPE_KEY,
        REF_EXCLUSIONS_KEY,
    ] {
        if let Some(value) = parent.get(key) {
            inputs.insert(key.to_string(), value.clone());
        }
    }
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn with_workflow_name(
    mut inputs: InputState,
    workflow: &Value,
    path: &str,
) -> InputState {
    let name = workflow
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or(path);
    inputs.insert(
        WORKFLOW_KEY.to_string(),
        StaticValue::String(name.to_string()),
    );
    inputs
}

pub(super) fn with_event(event: &GithubEventContext, mut inputs: InputState) -> InputState {
    inputs.insert(
        EVENT_NAME_KEY.to_string(),
        StaticValue::String(event.name.clone()),
    );
    inputs.insert(
        EVENT_ACTION_KEY.to_string(),
        StaticValue::String(match &event.action {
            GithubEventAction::Missing => String::new(),
            GithubEventAction::Known(action) => action.clone(),
        }),
    );
    if event.name == "pull_request"
        && matches!(&event.action, GithubEventAction::Known(action) if action == "synchronize")
    {
        inputs.insert(
            PULL_REQUEST_MERGED_KEY.to_string(),
            StaticValue::Bool(false),
        );
    }
    match &event.reference {
        GithubRef::Exact(reference) => {
            inputs.insert(REF_KEY.to_string(), StaticValue::String(reference.clone()));
            if let Some(kind) = exact_ref_kind(reference) {
                inputs.insert(
                    REF_KIND_KEY.to_string(),
                    StaticValue::String(kind.to_string()),
                );
                inputs.insert(
                    REF_SHAPE_KEY.to_string(),
                    StaticValue::String(kind.to_string()),
                );
            }
            if let Some(name) = exact_ref_name(reference) {
                inputs.insert(
                    REF_NAME_KEY.to_string(),
                    StaticValue::String(name.to_string()),
                );
            }
        }
        GithubRef::UnknownExcluding(references) => {
            inputs.insert(
                REF_KIND_KEY.to_string(),
                StaticValue::String("branch".to_string()),
            );
            inputs.insert(
                REF_SHAPE_KEY.to_string(),
                StaticValue::String("branch".to_string()),
            );
            inputs.insert(
                REF_EXCLUSIONS_KEY.to_string(),
                StaticValue::Sequence(
                    references
                        .iter()
                        .cloned()
                        .map(StaticValue::String)
                        .collect(),
                ),
            );
        }
        GithubRef::UnknownBranch => {
            inputs.insert(
                REF_KIND_KEY.to_string(),
                StaticValue::String("branch".to_string()),
            );
            inputs.insert(
                REF_SHAPE_KEY.to_string(),
                StaticValue::String("branch".to_string()),
            );
        }
        GithubRef::PullRequestMerge => {
            inputs.insert(
                REF_KIND_KEY.to_string(),
                StaticValue::String("branch".to_string()),
            );
            inputs.insert(
                REF_SHAPE_KEY.to_string(),
                StaticValue::String("pull-request-merge".to_string()),
            );
        }
        GithubRef::Unknown => {}
    }
    match &event.base_reference {
        GithubRef::Exact(reference) => {
            if let Some(name) = exact_ref_name(reference) {
                inputs.insert(
                    BASE_REF_KEY.to_string(),
                    StaticValue::String(name.to_string()),
                );
            }
        }
        GithubRef::Unknown
            if !matches!(event.name.as_str(), "pull_request" | "pull_request_target") =>
        {
            inputs.insert(BASE_REF_KEY.to_string(), StaticValue::String(String::new()));
        }
        GithubRef::Unknown
        | GithubRef::UnknownBranch
        | GithubRef::UnknownExcluding(_)
        | GithubRef::PullRequestMerge => {}
    }
    if !matches!(event.name.as_str(), "pull_request" | "pull_request_target") {
        inputs.insert(HEAD_REF_KEY.to_string(), StaticValue::String(String::new()));
    }
    inputs
}

fn exact_ref_name(reference: &str) -> Option<&str> {
    reference
        .strip_prefix("refs/heads/")
        .or_else(|| reference.strip_prefix("refs/tags/"))
}

fn exact_ref_kind(reference: &str) -> Option<&'static str> {
    reference
        .strip_prefix("refs/heads/")
        .map(|_| "branch")
        .or_else(|| reference.strip_prefix("refs/tags/").map(|_| "tag"))
}
