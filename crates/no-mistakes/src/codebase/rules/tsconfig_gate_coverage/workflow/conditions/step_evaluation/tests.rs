use super::super::{continue_on_error_value, EnvironmentState, InputState, StaticBool};
use serde_yaml::Value;

#[test]
fn continue_on_error_requires_a_static_boolean_to_change_the_default() {
    for (yaml, expected) in [
        ("run: tsc", StaticBool::False),
        ("continue-on-error: true", StaticBool::True),
        (
            "continue-on-error: '${{ vars.allow_failure }}'",
            StaticBool::Unknown,
        ),
    ] {
        let step: Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            continue_on_error_value(&step, &InputState::new(), &EnvironmentState::default()),
            expected,
            "{yaml}"
        );
    }
}
