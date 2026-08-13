use super::*;

#[test]
fn checkout_root_rejects_non_mapping_bindings() {
    assert!(!checkout_root_is_available(
        Some(&Value::Bool(true)),
        &InputState::new(),
        &EnvironmentState::default(),
    ));
}

#[test]
fn empty_repository_binding_keeps_the_current_repository_checkout_available() {
    let inputs = InputState::new();
    let environment = EnvironmentState::default();
    for repository in ["''", "\"${{ '' }}\""] {
        let bindings: Value = serde_yaml::from_str(&format!("repository: {repository}")).unwrap();
        assert!(checkout_root_is_available(
            Some(&bindings),
            &inputs,
            &environment
        ));
    }
    for repository in ["other/repository", "'${{ github.repository }}'"] {
        let bindings: Value = serde_yaml::from_str(&format!("repository: {repository}")).unwrap();
        assert!(!checkout_root_is_available(
            Some(&bindings),
            &inputs,
            &environment
        ));
    }
}
