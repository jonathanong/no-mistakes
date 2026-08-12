use super::{step_working_directory, EnvironmentState, InputState};
use serde_yaml::Value;

#[test]
fn dynamic_step_directory_cannot_fall_through_to_a_later_run_step() {
    let step: Value = serde_yaml::from_str("working-directory: '${{ vars.dir }}'").unwrap();
    assert_eq!(
        step_working_directory(
            &step,
            &InputState::new(),
            &EnvironmentState::default(),
            &None,
        ),
        None
    );
}
