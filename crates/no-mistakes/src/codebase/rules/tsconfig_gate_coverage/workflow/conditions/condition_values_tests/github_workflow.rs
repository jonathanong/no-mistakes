use super::{condition_value, EnvironmentState, InputState, StaticBool, StaticValue};

#[test]
fn github_workflow_value_resolves_from_the_activation_input() {
    let inputs = InputState::from([(
        "\0github.workflow".into(),
        StaticValue::String("Checks".into()),
    )]);
    assert_eq!(
        condition_value(
            "github.workflow",
            &inputs,
            &EnvironmentState::default(),
            StaticBool::True,
        ),
        Some(StaticValue::String("Checks".into()))
    );
}
