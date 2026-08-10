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
