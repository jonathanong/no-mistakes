use super::*;

#[test]
fn source_change_event_contexts_select_only_synchronize_activities() {
    for (yaml, event, expected) in [
        ("on: push", "push", vec![("push".to_string(), None)]),
        (
            "on: pull_request",
            "pull_request",
            vec![("pull_request".to_string(), Some("synchronize".to_string()))],
        ),
        (
            "on:\n  pull_request_target:\n    types: [opened, synchronize]",
            "pull_request_target",
            vec![(
                "pull_request_target".to_string(),
                Some("synchronize".to_string()),
            )],
        ),
        (
            "on:\n  pull_request:\n    types: [opened, closed]",
            "pull_request",
            vec![],
        ),
        (
            "on:\n  pull_request:\n    types: [synchronize]\n    branches-ignore: ['**']",
            "pull_request",
            vec![],
        ),
        (
            "on:\n  push:\n    branches-ignore: ['**']",
            "push",
            vec![],
        ),
        (
            "on:\n  pull_request:\n    types: [synchronize]\n    branches: [release/**, '!**']",
            "pull_request",
            vec![],
        ),
        // `release/**` does not exclude every branch, so source changes can
        // still activate this workflow on another branch.
        (
            "on:\n  pull_request:\n    types: [synchronize]\n    branches-ignore: [release/**]",
            "pull_request",
            vec![("pull_request".to_string(), Some("synchronize".to_string()))],
        ),
        // A positive pattern after `!**` re-includes matching branches.
        (
            "on:\n  pull_request:\n    types: [synchronize]\n    branches: [release/**, '!**', main]",
            "pull_request",
            vec![("pull_request".to_string(), Some("synchronize".to_string()))],
        ),
        // The later exact negative pattern excludes the branch re-included by `main`.
        (
            "on:\n  pull_request:\n    types: [synchronize]\n    branches: ['!**', main, '!main']",
            "pull_request",
            vec![],
        ),
    ] {
        let workflow: Value = serde_yaml::from_str(yaml).unwrap();
        let actual = source_change_event_contexts(&workflow, event)
            .into_iter()
            .map(|context| match context.action {
                super::super::model::GithubEventAction::Missing => (context.name, None),
                super::super::model::GithubEventAction::Known(action) => {
                    (context.name, Some(action))
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{yaml}");
    }
}
