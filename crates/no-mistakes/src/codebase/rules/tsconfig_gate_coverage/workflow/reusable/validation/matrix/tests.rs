use super::*;

fn job(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn uncertain_or_malformed_matrices_fail_open() {
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix:\n    target:\n      - [nested]"
    )));
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix:\n    target: [linux]\n    exclude: invalid"
    )));
    assert!(!zero_instance_matrix(&job(
        "strategy:\n  matrix: '${{ fromJSON(needs.setup.outputs.matrix) }}'"
    )));
}
