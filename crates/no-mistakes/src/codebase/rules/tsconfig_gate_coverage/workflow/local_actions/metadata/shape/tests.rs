use super::*;

#[test]
fn execution_shape_rejects_unknown_runtime_kinds() {
    let runs: Value = serde_yaml::from_str("using: wasm\nmain: action.wasm").unwrap();
    assert!(!runs_shape_valid(runs.as_mapping().unwrap(), "wasm"));
}

#[test]
fn action_metadata_rejects_invalid_shapes() {
    assert!(!action_inputs_valid(Some(
        &serde_yaml::from_str("1: {description: x}").unwrap()
    )));
    assert!(!action_inputs_valid(Some(
        &serde_yaml::from_str("name: not-a-map").unwrap()
    )));
    assert!(!outputs_valid(
        Some(&serde_yaml::from_str("out: {description: x, value: []}\n").unwrap()),
        true
    ));
    assert!(!branding_valid(Some(
        &serde_yaml::from_str("icon: activity\ncolor: neon").unwrap()
    )));
    let docker: Value =
        serde_yaml::from_str("using: docker\nimage: docker://example\nargs: not-a-list\n").unwrap();
    assert!(!runs_shape_valid(docker.as_mapping().unwrap(), "docker"));
    let env: Value =
        serde_yaml::from_str("using: docker\nimage: docker://example\nenv: not-a-map\n").unwrap();
    assert!(!runs_shape_valid(env.as_mapping().unwrap(), "docker"));
}

#[test]
fn action_metadata_rejects_duplicate_and_non_string_fields() {
    assert!(!action_inputs_valid(Some(
        &serde_yaml::from_str("Name: {description: x}\nname: {description: y}").unwrap()
    )));
    assert!(!outputs_valid(
        Some(&serde_yaml::from_str("1: {description: x}\n").unwrap()),
        false
    ));
    assert!(!outputs_valid(
        Some(&serde_yaml::from_str("out: not-a-map\n").unwrap()),
        false
    ));
    assert!(!branding_valid(Some(
        &serde_yaml::from_str("icon: []\n").unwrap()
    )));
    let args: Value =
        serde_yaml::from_str("using: docker\nimage: docker://example\nargs:\n  - 1\n").unwrap();
    assert!(!runs_shape_valid(args.as_mapping().unwrap(), "docker"));
    let env: Value =
        serde_yaml::from_str("using: docker\nimage: docker://example\nenv:\n  1: value\n").unwrap();
    assert!(!runs_shape_valid(env.as_mapping().unwrap(), "docker"));
}
