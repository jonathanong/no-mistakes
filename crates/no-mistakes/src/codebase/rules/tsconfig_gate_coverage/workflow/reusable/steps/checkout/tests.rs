use super::*;

#[test]
fn checkout_root_rejects_non_mapping_bindings() {
    assert!(!checkout_root_is_available(
        Some(&Value::Bool(true)),
        &InputState::new(),
        &EnvironmentState::default(),
    ));
}
