use super::*;

#[test]
fn missing_runner_cannot_imply_a_windows_default() {
    assert!(!runs_on_can_default_to_windows(&Value::Null));
}
