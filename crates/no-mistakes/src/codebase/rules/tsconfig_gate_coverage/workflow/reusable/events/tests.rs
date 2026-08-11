use super::*;

#[test]
fn pull_request_events_must_include_synchronize_when_types_are_explicit() {
    for (yaml, event, expected) in [
        ("on: push", "push", true),
        ("on: pull_request", "pull_request", true),
        ("on:\n  pull_request:\n", "pull_request", true),
        (
            "on:\n  pull_request:\n    types: [synchronize]",
            "pull_request",
            true,
        ),
        (
            "on:\n  pull_request_target:\n    types: [opened, synchronize]",
            "pull_request_target",
            true,
        ),
        (
            "on:\n  pull_request:\n    types: [opened, closed]",
            "pull_request",
            false,
        ),
    ] {
        let workflow: Value = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            source_change_event_eligible(&workflow, event),
            expected,
            "{yaml}"
        );
    }
}

#[test]
fn direct_pull_request_actions_are_sorted_and_event_specific() {
    let workflow: Value = serde_yaml::from_str(
        "on:\n  pull_request:\n    types: [synchronize, opened, synchronize]\n  pull_request_target:\n    types: [closed]",
    )
    .unwrap();

    assert_eq!(
        direct_event_actions(&workflow, "pull_request"),
        vec![Some("opened".to_string()), Some("synchronize".to_string())]
    );
    assert_eq!(
        direct_event_actions(&workflow, "pull_request_target"),
        vec![None]
    );
    assert_eq!(direct_event_actions(&workflow, "push"), vec![None]);
}
