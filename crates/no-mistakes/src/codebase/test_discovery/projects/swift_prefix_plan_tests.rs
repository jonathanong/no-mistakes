use crate::test_support::materialize_saved_fixture;
use crate::tests::targets::commands_for_plan;
use crate::tests::{PlanArgs, TestFramework};
use std::path::{Path, PathBuf};

fn plan(root: &Path) -> crate::tests::TestPlan {
    crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Swift),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: vec![root.join("clients/core-extra/Tests/ExtraTests/ExtraTests.swift")],
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: None,
        environment: "pre-push".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: Some(false),
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
        include_glob: Vec::new(),
    })
    .unwrap()
}

/// The saved package list puts `clients/core` first. Core-extra tests must
/// still render the longer package, not core's runner override.
#[test]
fn swift_prefix_package_renders_the_longer_package_path() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-prefix-packages");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let plan = plan(&root);
    let commands = commands_for_plan(&plan);

    assert!(
        commands
            .iter()
            .any(|command| { command.contains("swift test --package-path clients/core-extra") }),
        "{commands:#?}\n{plan:#?}"
    );
    assert!(
        !commands.iter().any(|command| {
            command.contains("/opt/core-only-swift")
                || command.contains("--package-path clients/core ")
                || command.ends_with("--package-path clients/core")
        }),
        "{commands:#?}\n{plan:#?}"
    );
}
