use super::*;

fn labels(yaml: &str) -> Option<Vec<String>> {
    let job: Value = serde_yaml::from_str(yaml).unwrap();
    static_runner_labels(job.as_mapping(), &InputState::new())
}

#[test]
fn runner_selection_rejects_unresolved_or_malformed_runtime_labels() {
    assert_eq!(
        labels("runs-on: [self-hosted, linux]"),
        Some(vec!["self-hosted".to_string(), "linux".to_string()])
    );
    assert_eq!(
        labels("runs-on: {group: ubuntu-runners, labels: [ubuntu-latest]}"),
        Some(vec!["ubuntu-latest".to_string()])
    );
    for yaml in [
        "name: missing",
        "runs-on: []",
        "runs-on: 42",
        "runs-on: ['${{ github.ref }}']",
        "runs-on: {group: '${{ github.ref }}'}",
        "runs-on: {labels: '${{ github.ref }}'}",
        "runs-on: {labels: []}",
        "runs-on: '${{ }}'",
    ] {
        assert!(labels(yaml).is_none(), "{yaml}");
    }
}
