use super::*;

fn selection(yaml: &str) -> Option<StaticRunnerSelection> {
    let job: Value = serde_yaml::from_str(yaml).unwrap();
    static_runner_selection(job.as_mapping(), &InputState::new())
}

#[test]
fn runner_selection_rejects_unresolved_or_malformed_runtime_labels() {
    assert_eq!(
        selection("runs-on: [self-hosted, linux]"),
        Some(StaticRunnerSelection {
            group: None,
            labels: vec!["self-hosted".to_string(), "linux".to_string()],
        })
    );
    assert_eq!(
        selection("runs-on: {group: ubuntu-runners, labels: [ubuntu-latest]}"),
        Some(StaticRunnerSelection {
            group: Some("ubuntu-runners".to_string()),
            labels: vec!["ubuntu-latest".to_string()],
        })
    );
    assert_eq!(
        selection("runs-on: {group: ubuntu-latest}"),
        Some(StaticRunnerSelection {
            group: Some("ubuntu-latest".to_string()),
            labels: Vec::new(),
        })
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
        assert!(selection(yaml).is_none(), "{yaml}");
    }
}
