use super::{direct_inputs, event::HEAD_REF_KEY, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    GithubEventContext, GithubRef,
};

#[test]
fn non_pull_request_activations_have_an_empty_head_ref() {
    let inputs = direct_inputs(
        None,
        &GithubEventContext::with_ref("push", GithubRef::UnknownBranch),
    )
    .unwrap();
    assert_eq!(
        inputs.get(HEAD_REF_KEY),
        Some(&StaticValue::String(String::new()))
    );
}

#[test]
fn exact_tag_refs_expose_name_and_kind() {
    let inputs = super::event::with_event(
        &GithubEventContext::with_ref("push", GithubRef::Exact("refs/tags/v1.2.3".into())),
        super::InputState::new(),
    );
    assert_eq!(
        super::event_ref_name_value(&inputs),
        Some(StaticValue::String("v1.2.3".into()))
    );
    assert_eq!(
        super::event_ref_type_value(&inputs),
        Some(StaticValue::String("tag".into()))
    );
    assert_eq!(
        super::event_head_ref_value(&inputs),
        Some(StaticValue::String(String::new()))
    );
}
