use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
use crate::tests::{PlanArgs, PlanIncompletenessKind, TestFramework};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture_root() -> (tempfile::TempDir, PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dependency-baseline-incomplete/dotnet");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

fn plan(root: &Path, base: Option<&str>, changed_file: Vec<PathBuf>) -> crate::tests::TestPlan {
    crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Dotnet),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: base.map(str::to_string),
        head: None,
        from_git_diff: None,
        changed_file,
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
        include_comment: true,
        include_glob: Vec::new(),
    })
    .unwrap()
}

/// The three .NET inputs are different files and one missing-baseline
/// condition, so `plan_finish` records a single `dependency-baseline-missing`.
#[test]
fn dotnet_missing_dependency_baseline_is_incomplete() {
    let (_fixture, root) = fixture_root();
    let plan = plan(
        &root,
        None,
        vec![
            root.join("App.csproj"),
            root.join("Directory.Packages.props"),
            root.join("packages.lock.json"),
        ],
    );
    let json = serde_json::to_value(&plan).unwrap();

    assert_eq!(
        json["incomplete"],
        serde_json::json!([{
            "kind": PlanIncompletenessKind::DependencyBaselineMissing.as_str(),
            "files": ["App.csproj", "Directory.Packages.props", "packages.lock.json"]
        }]),
        "{plan:#?}"
    );
    assert_eq!(
        plan.warnings
            .iter()
            .filter(|warning| warning.r#type == "dotnet-dependency-no-baseline")
            .map(|warning| warning.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "App.csproj",
            "Directory.Packages.props",
            "packages.lock.json",
        ]),
        "{plan:#?}"
    );
    assert!(!plan.fallback_triggered, "{plan:#?}");
    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    let comment = plan.comment.as_deref().expect("comment");
    assert!(comment.contains("dependency-baseline-missing"));
    assert!(comment.contains("`App.csproj`"));
    assert!(comment.contains("`Directory.Packages.props`"));
    assert!(comment.contains("`packages.lock.json`"));
}

/// A readable baseline is complete coverage. The version bump is a saved
/// `after/` copy, not a test-written file, so the with-base path cannot emit
/// `dotnet-dependency-no-baseline` or `incomplete`.
#[test]
fn dotnet_dependency_baseline_omits_incomplete_when_base_is_present() {
    let (_fixture, root) = fixture_root();
    git_init(&root);
    git_commit_all(&root, "base");
    std::fs::copy(
        root.join("after/Directory.Packages.props"),
        root.join("Directory.Packages.props"),
    )
    .unwrap();
    let plan = plan(
        &root,
        Some("HEAD"),
        vec![root.join("Directory.Packages.props")],
    );
    let json = serde_json::to_value(&plan).unwrap();

    assert!(json.get("incomplete").is_none(), "{plan:#?}");
    assert!(
        !plan
            .warnings
            .iter()
            .any(|warning| warning.r#type == "dotnet-dependency-no-baseline"),
        "{plan:#?}"
    );
}
