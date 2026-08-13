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
