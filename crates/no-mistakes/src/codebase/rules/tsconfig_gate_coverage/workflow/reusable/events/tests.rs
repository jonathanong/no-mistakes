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

#[test]
fn exact_ref_filters_produce_fully_qualified_ref_contexts() {
    for (yaml, event, expected) in [
        (
            "on:\n  push:\n    branches: [main, dev]",
            "push",
            vec!["refs/heads/dev", "refs/heads/main"],
        ),
        ("on:\n  push:\n    tags: [v1]", "push", vec!["refs/tags/v1"]),
        (
            "on:\n  pull_request_target:\n    types: [synchronize]\n    branches: [main]",
            "pull_request_target",
            vec!["refs/heads/main"],
        ),
    ] {
        let workflow: Value = serde_yaml::from_str(yaml).unwrap();
        let actual = source_change_event_contexts(&workflow, event)
            .into_iter()
            .filter_map(|context| match context.reference {
                GithubRef::Exact(reference) => Some(reference),
                GithubRef::PullRequestMerge | GithubRef::Unknown => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{yaml}");
    }

    let workflow: Value =
        serde_yaml::from_str("on:\n  push:\n    branches: ['release/**']\n    tags-ignore: [v0]")
            .unwrap();
    assert!(source_change_event_contexts(&workflow, "push")
        .iter()
        .all(|context| matches!(context.reference, GithubRef::Unknown)));
}
