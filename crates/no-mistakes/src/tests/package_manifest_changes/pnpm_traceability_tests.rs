use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
use crate::tests::{PlanArgs, TestFramework};
use std::path::PathBuf;

#[test]
fn untraceable_lockfile_dependency_falls_back_even_when_another_dependency_is_reachable() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/pnpm-traceability/fixture");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    git_init(&root);
    git_commit_all(&root, "base");
    let lockfile = root.join("pnpm-lock.yaml");
    std::fs::copy(root.join("changes/pnpm-lock.yaml"), &lockfile).unwrap();

    let plan = crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.clone(),
        config: None,
        tsconfig: None,
        base: Some("HEAD".to_string()),
        head: None,
        from_git_diff: None,
        changed_file: vec![lockfile],
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: None,
        environment: "prePush".to_string(),
        limit_percent: None,
        limit_files: None,
        global_config_fallback: Some(true),
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
        include_glob: Vec::new(),
    })
    .unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan.warnings.iter().any(|warning| {
        warning.r#type == "package-dependency-untraceable"
            && warning.message.contains("untraceable-leaf")
    }));
}
