use super::{InputState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    GithubEventAction, GithubEventContext, GithubRef,
};

pub(in super::super) const EVENT_NAME_KEY: &str = "\0github.event_name";
pub(in super::super) const EVENT_ACTION_KEY: &str = "\0github.event.action";
pub(in super::super) const REF_KEY: &str = "\0github.ref";
pub(in super::super) const REF_NAME_KEY: &str = "\0github.ref_name";
pub(in super::super) const REF_KIND_KEY: &str = "\0github.ref.kind";
pub(in super::super) const REF_EXCLUSIONS_KEY: &str = "\0github.ref.exclusions";

pub(in super::super) fn event_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_NAME_KEY).cloned()
}
pub(in super::super) fn event_action_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(EVENT_ACTION_KEY).cloned()
}
pub(in super::super) fn event_ref_name_value(inputs: &InputState) -> Option<StaticValue> {
    inputs.get(REF_NAME_KEY).cloned()
}

pub(super) fn copy_event_inputs(parent: &InputState, inputs: &mut InputState) {
    for key in [
        EVENT_NAME_KEY,
        EVENT_ACTION_KEY,
        REF_KEY,
        REF_NAME_KEY,
        REF_KIND_KEY,
        REF_EXCLUSIONS_KEY,
    ] {
        if let Some(value) = parent.get(key) {
            inputs.insert(key.to_string(), value.clone());
        }
    }
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
    match &event.reference {
        GithubRef::Exact(reference) => {
            inputs.insert(REF_KEY.to_string(), StaticValue::String(reference.clone()));
            if let Some(name) = exact_ref_name(reference) {
                inputs.insert(
                    REF_NAME_KEY.to_string(),
                    StaticValue::String(name.to_string()),
                );
            }
        }
        GithubRef::UnknownExcluding(references) => {
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
        GithubRef::PullRequestMerge => {
            inputs.insert(
                REF_KIND_KEY.to_string(),
                StaticValue::String("pull-request-merge".to_string()),
            );
        }
        GithubRef::Unknown => {}
    }
    inputs
}

fn exact_ref_name(reference: &str) -> Option<&str> {
    reference
        .strip_prefix("refs/heads/")
        .or_else(|| reference.strip_prefix("refs/tags/"))
}
