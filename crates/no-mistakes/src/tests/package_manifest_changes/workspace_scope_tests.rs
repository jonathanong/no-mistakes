use crate::tests::{PlanArgs, TestFramework};
use std::path::PathBuf;

fn package_manifest_plan_fixture() -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-plan/fixture");
    crate::test_support::materialize_saved_fixture(&source)
}

fn workspace_plan(
    root: &std::path::Path,
    manifest: PathBuf,
    fallback: bool,
) -> crate::tests::TestPlan {
    crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.to_path_buf(),
        base: Some("HEAD".to_string()),
        changed_file: vec![manifest],
        environment: "prePush".to_string(),
        global_config_fallback: Some(fallback),
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
    .unwrap()
}

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

#[test]
fn deleted_workspace_manifest_warns_and_obeys_fallback_policy() {
    for fallback in [false, true] {
        let fixture = package_manifest_plan_fixture();
        let root = fixture.path().canonicalize().unwrap();
        let manifest = root.join("workspaces/a/package.json");
        crate::test_support::git_init(&root);
        crate::test_support::git_commit_all(&root, "base");
        std::fs::remove_file(&manifest).unwrap();

        let plan = workspace_plan(&root, manifest, fallback);
        assert_eq!(plan.fallback_triggered, fallback, "{plan:#?}");
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.r#type == "package-manifest-no-baseline"));
    }
}

#[test]
fn normalized_workspace_glob_scopes_a_deleted_manifest() {
    let fixture = package_manifest_plan_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let manifest = root.join("workspaces/a/package.json");
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    std::fs::copy(
        root.join("changes/pnpm-workspace-dot-path.yaml"),
        root.join("pnpm-workspace.yaml"),
    )
    .unwrap();
    std::fs::remove_file(&manifest).unwrap();

    let plan = workspace_plan(&root, manifest, false);
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "package-manifest-no-baseline"));
}

#[test]
fn malformed_workspace_manifest_warns_and_obeys_fallback_policy() {
    for fallback in [false, true] {
        let fixture = package_manifest_plan_fixture();
        let root = fixture.path().canonicalize().unwrap();
        let manifest = root.join("workspaces/a/package.json");
        crate::test_support::git_init(&root);
        crate::test_support::git_commit_all(&root, "base");
        std::fs::copy(root.join("changes/workspace-a-malformed.json"), &manifest).unwrap();

        let plan = workspace_plan(&root, manifest, fallback);
        assert_eq!(plan.fallback_triggered, fallback, "{plan:#?}");
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.r#type == "package-manifest-malformed"));
    }
}
