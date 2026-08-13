use super::*;

#[test]
fn execution_shape_rejects_unknown_runtime_kinds() {
    let runs: Value = serde_yaml::from_str("using: wasm\nmain: action.wasm").unwrap();
    assert!(!runs_shape_valid(runs.as_mapping().unwrap(), "wasm"));
}
