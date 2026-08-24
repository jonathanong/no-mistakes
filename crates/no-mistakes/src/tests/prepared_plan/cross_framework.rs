use super::*;

#[test]
fn vitest_keeps_dotnet_dependency_files_in_its_changed_inventory() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dotnet-dependency-diff");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    let project = root.join("base.csproj");
    std::fs::copy(root.join("project-add.csproj"), &project).unwrap();
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.base = Some("HEAD".to_string());
    args.changed_file = vec![project.clone()];

    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();

    assert!(prepared.dotnet_dependency_analysis.handles(&project));
    assert!(prepared
        .planning_changed_files(Some(TestFramework::Vitest))
        .contains(&project));
}

#[test]
fn shared_fanout_projects_native_files_for_each_requested_framework() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dotnet-dependency-diff");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    let project = root.join("base.csproj");
    std::fs::copy(root.join("project-add.csproj"), &project).unwrap();
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.framework = None;
    args.base = Some("HEAD".to_string());
    args.changed_file = vec![project.clone()];

    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();

    assert!(prepared
        .planning_changed_files(Some(TestFramework::Vitest))
        .contains(&project));
    assert!(!prepared
        .planning_changed_files(Some(TestFramework::Dotnet))
        .contains(&project));
}

#[test]
fn dotnet_keeps_javascript_dependency_files_without_js_fallback() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/package-manifest-plan/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    let manifest = root.join("workspaces/a/package.json");
    std::fs::copy(root.join("changes/workspace-a-package.json"), &manifest).unwrap();
    let mut args = framework_args(&root, TestFramework::Dotnet);
    args.base = Some("HEAD".to_string());
    args.changed_file = vec![manifest.clone()];
    args.global_config_fallback = Some(true);

    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    assert!(prepared.package_manifest_analysis.handles(&manifest));
    assert!(prepared
        .planning_changed_files(Some(TestFramework::Dotnet))
        .contains(&manifest));
    assert!(!prepared.is_dependency_only_manifest(&manifest, Some(TestFramework::Dotnet)));
    let plan =
        crate::tests::plan::generate_plan_with_prepared(prepared.args(), &prepared, None).unwrap();
    assert!(!plan.fallback_triggered, "{plan:#?}");
}

#[test]
fn vitest_keeps_swift_dependency_manifests_available_to_its_triggers() {
    let (fixture, root, manifest) = swift_fixture();
    std::fs::copy(root.join("changes/core-added-dependency.swift"), &manifest).unwrap();
    let prepared = prepared_vitest_request(&root, manifest.clone());

    assert!(prepared.swift_manifest_analysis.handles(&manifest));
    assert!(prepared
        .swift_manifest_analysis
        .dependency_only_files()
        .contains(&manifest));
    assert!(prepared
        .planning_changed_files(Some(TestFramework::Vitest))
        .contains(&manifest));
    assert!(!prepared.is_dependency_only_manifest(&manifest, Some(TestFramework::Vitest)));
    let plan =
        crate::tests::plan::generate_plan_with_prepared(prepared.args(), &prepared, None).unwrap();
    assert!(!plan.fallback_triggered, "{plan:#?}");
    drop(fixture);
}

#[test]
fn swift_diagnostic_does_not_force_an_unrelated_vitest_fallback() {
    let (fixture, root, manifest) = swift_fixture();
    let dynamic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-manifest-diff/fixture/dynamic.swift");
    std::fs::copy(dynamic, &manifest).unwrap();
    let mut args = framework_args(&root, TestFramework::Vitest);
    args.base = Some("HEAD".to_string());
    args.changed_file = vec![manifest];
    args.global_config_fallback = Some(true);

    let prepared = PreparedTestPlanRequest::prepare(&args).unwrap();
    assert!(prepared.swift_manifest_analysis.fallback_triggered);
    let plan =
        crate::tests::plan::generate_plan_with_prepared(prepared.args(), &prepared, None).unwrap();

    assert!(!plan.fallback_triggered, "{plan:#?}");
    drop(fixture);
}

fn swift_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-native-topology/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::test_support::git_init(&root);
    crate::test_support::git_commit_all(&root, "base");
    let manifest = root.join("swift-clients/core/Package.swift");
    (fixture, root, manifest)
}

fn prepared_vitest_request(root: &Path, changed: PathBuf) -> PreparedTestPlanRequest {
    let mut args = framework_args(root, TestFramework::Vitest);
    args.base = Some("HEAD".to_string());
    args.changed_file = vec![changed];
    args.global_config_fallback = Some(true);
    PreparedTestPlanRequest::prepare(&args).unwrap()
}
