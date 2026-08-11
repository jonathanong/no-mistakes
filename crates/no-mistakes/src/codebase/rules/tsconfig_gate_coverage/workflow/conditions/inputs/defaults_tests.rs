use super::*;
use crate::codebase::workflow_topology::model::WorkflowCallInput;

#[test]
fn reusable_defaults_resolve_the_caller_event_and_prior_inputs() {
    let contract = WorkflowCallContract {
        inputs: BTreeMap::from([
            (
                "a-enabled".to_string(),
                WorkflowCallInput {
                    input_type: Some(WorkflowCallInputType::Boolean),
                    required: false,
                    default: Some(JsonScalar::Text(
                        "${{ github.event_name == 'schedule' }}".to_string(),
                    )),
                    description: None,
                },
            ),
            (
                "follows".to_string(),
                WorkflowCallInput {
                    input_type: Some(WorkflowCallInputType::Boolean),
                    required: false,
                    default: Some(JsonScalar::Text("${{ inputs['a-enabled'] }}".to_string())),
                    description: None,
                },
            ),
        ]),
        ..WorkflowCallContract::default()
    };
    let caller = direct_inputs(
        None,
        &crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::GithubEventContext::without_action("schedule"),
    )
    .unwrap();
    let job: Value = serde_yaml::from_str("uses: ./.github/workflows/checks.yml").unwrap();

    assert_eq!(
        callee_inputs(Some(&contract), &job, &caller),
        Some(InputState::from([
            ("a-enabled".to_string(), StaticValue::Bool(true)),
            ("follows".to_string(), StaticValue::Bool(true)),
            (
                "\0github.event_name".to_string(),
                StaticValue::String("schedule".to_string()),
            ),
            (
                "\0github.event.action".to_string(),
                StaticValue::String(String::new()),
            ),
        ]))
    );
}
