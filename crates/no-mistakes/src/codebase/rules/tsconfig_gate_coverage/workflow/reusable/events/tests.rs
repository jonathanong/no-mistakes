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
        ("on:\n  push:\n    branches-ignore: ['**']", "push", vec![]),
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
fn tag_only_pushes_do_not_model_source_change_activations() {
    let workflow: Value = serde_yaml::from_str("on:\n  push:\n    tags: [v1]").unwrap();

    assert!(source_change_event_contexts(&workflow, "push").is_empty());
}

#[test]
fn exact_ref_filters_produce_fully_qualified_ref_contexts() {
    for (yaml, event, expected) in [
        (
            "on:\n  push:\n    branches: [main, dev]",
            "push",
            vec!["refs/heads/dev", "refs/heads/main"],
        ),
        // Tag-only pushes do not carry source-path changes.
        ("on:\n  push:\n    tags: [v1]", "push", vec![]),
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
                GithubRef::UnknownExcluding(_)
                | GithubRef::UnknownBranch
                | GithubRef::PullRequestMerge
                | GithubRef::Unknown => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{yaml}");
    }

    let workflow: Value =
        serde_yaml::from_str("on:\n  push:\n    branches: ['release/**']\n    tags-ignore: [v0]")
            .unwrap();
    assert!(matches!(
        source_change_event_contexts(&workflow, "push").as_slice(),
        [context] if matches!(context.reference, GithubRef::UnknownBranch)
    ));

    let workflow: Value =
        serde_yaml::from_str("on:\n  push:\n    branches-ignore: [main, 'release/**']").unwrap();
    assert!(matches!(
        source_change_event_contexts(&workflow, "push").as_slice(),
        [context] if matches!(
            &context.reference,
            GithubRef::UnknownExcluding(excluded)
                if excluded == &std::collections::BTreeSet::from(["refs/heads/main".to_string()])
        )
    ));
}

#[test]
fn mixed_exact_and_wildcard_branch_filters_preserve_every_activation_alternative() {
    let workflow: Value =
        serde_yaml::from_str("on:\n  push:\n    branches: [main, 'release/**']").unwrap();
    let references = source_change_event_contexts(&workflow, "push")
        .into_iter()
        .map(|context| context.reference)
        .collect::<Vec<_>>();

    assert!(matches!(
        references.as_slice(),
        [GithubRef::Exact(reference), GithubRef::UnknownExcluding(excluded)]
            if reference == "refs/heads/main"
                && excluded == &BTreeSet::from(["refs/heads/main".to_string()])
    ));
}

#[test]
fn pull_request_merge_ref_retains_the_exact_base_ref() {
    let workflow: Value = serde_yaml::from_str(
        "on:\n  pull_request:\n    types: [synchronize]\n    branches: [main]",
    )
    .unwrap();

    assert!(matches!(
        source_change_event_contexts(&workflow, "pull_request").as_slice(),
        [context]
            if matches!(context.reference, GithubRef::PullRequestMerge)
                && matches!(
                    &context.base_reference,
                    GithubRef::Exact(reference) if reference == "refs/heads/main"
                )
    ));
}
