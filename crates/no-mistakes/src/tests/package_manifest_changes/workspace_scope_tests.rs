use crate::tests::{PlanArgs, TestFramework};
use std::path::PathBuf;

#[test]
fn nested_non_workspace_manifest_remains_available_to_broad_triggers() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-workspace-scope/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let manifest = root.join("examples/tool/package.json");
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    std::fs::copy(root.join("changes/tool-package.json"), &manifest).unwrap();

    let plan = crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.clone(),
        base: Some("HEAD".to_string()),
        changed_file: vec![manifest],
        environment: "prePush".to_string(),
        global_config_fallback: Some(true),
        config: None,
        tsconfig: None,
        head: None,
        from_git_diff: None,
        changed_files: None,
        diff: None,
        diff_stdin: false,
        diff_command: None,
        entrypoints: Vec::new(),
        entrypoint_symbols: Vec::new(),
        include_symbols: false,
        diff_content: None,
        limit_percent: None,
        limit_files: None,
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
        include_glob: Vec::new(),
    })
    .unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .fallback_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("examples project dependency changed")));
}
