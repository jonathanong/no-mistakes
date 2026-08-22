use super::grouped_execution_targets;
use crate::codebase::test_discovery::TestExecutionTarget;
use crate::tests::{Confidence, SelectedTest};

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
    let groups = grouped_execution_targets(&[
        selected(
            "pkg/tests.py",
            vec![target("python", Some("pkg"), &["python", "-m", "unittest"])],
        ),
        selected(
            "pkg/test_foo.py",
            vec![target("python", Some("pkg"), &["pytest"])],
        ),
    ]);

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
