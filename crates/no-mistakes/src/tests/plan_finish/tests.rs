use super::grouped_execution_targets;
use crate::codebase::test_discovery::TestExecutionTarget;
use crate::tests::{
    Confidence, PlanIncompleteness, SelectedTest, TestPlan, Warning, DEPENDENCY_BASELINE_MISSING,
};

fn target(runner: &str, project: Option<&str>, base: &[&str]) -> TestExecutionTarget {
    TestExecutionTarget {
        runner: runner.to_string(),
        config: None,
        workspace: false,
        project: project.map(str::to_string),
        base_command: base.iter().map(|s| (*s).to_string()).collect(),
        runner_args: Vec::new(),
    }
}

fn warning(kind: &str, file: &str) -> Warning {
    Warning {
        r#type: kind.to_string(),
        message: format!("{kind} for {file}"),
        file: file.to_string(),
        line: None,
    }
}

#[test]
fn finish_marks_missing_dependency_baselines_once_and_keeps_other_warnings() {
    let mut plan = TestPlan {
        warnings: vec![
            warning("package-manifest-no-baseline", "tooling/package.json"),
            warning("lockfile-no-baseline", "pnpm-lock.yaml"),
            warning("package-manifest-no-baseline", "package.json"),
            warning("package-manifest-no-baseline", "package.json"),
            warning("dynamic-import", "app/app.test.ts"),
        ],
        ..Default::default()
    };
    plan.finish(true, &[]);

    assert_eq!(
        plan.incomplete,
        vec![PlanIncompleteness {
            kind: DEPENDENCY_BASELINE_MISSING.to_string(),
            files: vec![
                "package.json".to_string(),
                "pnpm-lock.yaml".to_string(),
                "tooling/package.json".to_string(),
            ],
        }]
    );
    assert_eq!(plan.warnings.len(), 5);
    let comment = plan.comment.as_deref().expect("comment");
    assert!(comment.contains("dependency-baseline-missing"));
    assert!(comment.contains("`package.json`"));
    assert!(comment.contains("`--base <pre-change revision>`"));
    let json = serde_json::to_value(&plan).unwrap();
    assert_eq!(json["incomplete"][0]["kind"], "dependency-baseline-missing");
    assert_eq!(
        json["fallback_triggered"], false,
        "the marker must not flip fallback"
    );

    plan.finish(false, &[]);
    assert_eq!(plan.incomplete.len(), 1);
    // `finish(false)` does not regenerate or clear a comment from an earlier pass.
    assert!(plan
        .comment
        .as_deref()
        .is_some_and(|comment| { comment.contains("dependency-baseline-missing") }));
}

#[test]
fn finish_clears_a_stale_incomplete_marker_when_baselines_are_present() {
    let mut plan = TestPlan {
        incomplete: vec![PlanIncompleteness {
            kind: DEPENDENCY_BASELINE_MISSING.to_string(),
            files: vec!["package.json".to_string()],
        }],
        warnings: vec![warning("package-dependency-untraceable", "package.json")],
        ..Default::default()
    };
    plan.finish(false, &[]);
    assert!(plan.incomplete.is_empty(), "{plan:#?}");
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json.get("incomplete").is_none(), "{json}");
}

fn selected(file: &str, targets: Vec<TestExecutionTarget>) -> SelectedTest {
    SelectedTest {
        test_file: file.to_string(),
        confidence: Confidence::High,
        reasons: Vec::new(),
        targets,
    }
}

/// unittest-style `tests.py` and pytest files in the same package share a
/// runner/config/project tuple but must not collapse onto the first
/// `base_command`.
#[test]
fn mixed_python_runners_keep_separate_execution_targets() {
    let groups = grouped_execution_targets(
        &[
            selected(
                "pkg/tests.py",
                vec![target("python", Some("pkg"), &["python", "-m", "unittest"])],
            ),
            selected(
                "pkg/test_foo.py",
                vec![target("python", Some("pkg"), &["pytest"])],
            ),
        ],
        &[],
    );

    assert_eq!(groups.len(), 2);
    let mut commands: Vec<_> = groups.into_iter().map(|group| group.base_command).collect();
    commands.sort();
    assert_eq!(
        commands,
        vec![
            vec!["pytest".to_string()],
            vec![
                "python".to_string(),
                "-m".to_string(),
                "unittest".to_string()
            ],
        ]
    );
}

#[test]
fn nested_package_relative_runner_args_are_stripped_when_grouping() {
    let mut dart = target(
        "dart",
        None,
        &["dart", "pub", "--directory", "packages/app", "run", "test"],
    );
    dart.runner_args = vec!["test/user_test.dart".into()];
    dart.config = Some("packages/app".into());
    let groups = grouped_execution_targets(
        &[selected("packages/app/test/user_test.dart", vec![dart])],
        &[],
    );
    assert_eq!(groups.len(), 1);
    assert!(groups[0].runner_args.is_empty());
    assert_eq!(
        groups[0].test_files,
        vec!["packages/app/test/user_test.dart".to_string()]
    );
}

#[test]
fn path_prefixes_split_and_name_execution_targets() {
    let groups = grouped_execution_targets(
        &[
            selected(
                "swift-clients/core/Tests/A.swift",
                vec![target("swift", None, &["swift", "test"])],
            ),
            selected(
                "swift-clients/ui/Tests/B.swift",
                vec![target("swift", None, &["swift", "test"])],
            ),
        ],
        &["swift-clients/core".into(), "swift-clients/ui".into()],
    );
    assert_eq!(groups.len(), 2);
    let mut names: Vec<_> = groups.into_iter().map(|group| group.name).collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            Some("swift-clients/core".into()),
            Some("swift-clients/ui".into())
        ]
    );
}

#[test]
fn path_prefixes_name_only_swift_execution_targets() {
    let groups = grouped_execution_targets(
        &[selected(
            "swift-clients/core/web.test.ts",
            vec![target("vitest", None, &["vitest", "run"])],
        )],
        &["swift-clients/core".into()],
    );
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, None);
}

#[test]
fn distinct_selector_arguments_keep_execution_targets_separate() {
    let mut cargo_a = target("cargo", Some("app"), &["cargo", "test"]);
    cargo_a.runner_args = vec!["-p".into(), "app".into(), "--test".into(), "a".into()];
    let mut cargo_b = cargo_a.clone();
    cargo_b.runner_args[3] = "b".into();
    let mut swift_a = target("swift", Some("swift"), &["swift", "test"]);
    swift_a.runner_args = vec!["--filter".into(), "AlphaTests".into()];
    let mut swift_b = swift_a.clone();
    swift_b.runner_args[1] = "BetaTests".into();

    let groups = grouped_execution_targets(
        &[
            selected("tests/a.rs", vec![cargo_a]),
            selected("tests/b.rs", vec![cargo_b]),
            selected("Tests/Alpha.swift", vec![swift_a]),
            selected("Tests/Beta.swift", vec![swift_b]),
        ],
        &[],
    );

    assert_eq!(groups.len(), 4);
    assert!(groups.iter().any(|group| {
        group.runner == "cargo"
            && group.runner_args == ["-p", "app", "--test", "a"]
            && group.test_files == ["tests/a.rs"]
    }));
    assert!(groups.iter().any(|group| {
        group.runner == "cargo"
            && group.runner_args == ["-p", "app", "--test", "b"]
            && group.test_files == ["tests/b.rs"]
    }));
    assert!(groups.iter().any(|group| {
        group.runner == "swift"
            && group.runner_args == ["--filter", "AlphaTests"]
            && group.test_files == ["Tests/Alpha.swift"]
    }));
    assert!(groups.iter().any(|group| {
        group.runner == "swift"
            && group.runner_args == ["--filter", "BetaTests"]
            && group.test_files == ["Tests/Beta.swift"]
    }));
}

#[test]
fn identical_selector_arguments_still_combine_execution_targets() {
    let mut swift_a = target("swift", Some("swift"), &["swift", "test"]);
    swift_a.runner_args = vec!["--filter".into(), "SharedTests".into()];
    let swift_b = swift_a.clone();

    let groups = grouped_execution_targets(
        &[
            selected("Tests/SharedA.swift", vec![swift_a]),
            selected("Tests/SharedB.swift", vec![swift_b]),
        ],
        &[],
    );

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].runner_args, ["--filter", "SharedTests"]);
    assert_eq!(
        groups[0].test_files,
        ["Tests/SharedA.swift", "Tests/SharedB.swift"]
    );
}
