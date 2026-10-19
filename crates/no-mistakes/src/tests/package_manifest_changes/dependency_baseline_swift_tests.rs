use crate::test_support::materialize_saved_fixture;
use crate::tests::{PlanArgs, PlanIncompletenessKind, TestFramework};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture_root() -> (tempfile::TempDir, PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dependency-baseline-incomplete/swift");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

fn plan(root: &Path) -> crate::tests::TestPlan {
    crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Swift),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: None,
        head: None,
        from_git_diff: None,
        changed_file: vec![root.join("Package.swift"), root.join("Package.resolved")],
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

/// The two Swift inputs are different files and one missing-baseline
/// condition, so `plan_finish` records a single `dependency-baseline-missing`.
#[test]
fn swift_missing_dependency_baseline_is_incomplete() {
    let (_fixture, root) = fixture_root();
    let plan = plan(&root);
    let json = serde_json::to_value(&plan).unwrap();

    assert_eq!(
        json["incomplete"],
        serde_json::json!([{
            "kind": PlanIncompletenessKind::DependencyBaselineMissing.as_str(),
            "files": ["Package.resolved", "Package.swift"]
        }]),
        "{plan:#?}"
    );
    assert_eq!(
        plan.warnings
            .iter()
            .filter(|warning| {
                warning.r#type == "swift-manifest-no-baseline"
                    || warning.r#type == "swift-resolved-no-baseline"
            })
            .map(|warning| warning.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["Package.resolved", "Package.swift"]),
        "{plan:#?}"
    );
    assert!(!plan.fallback_triggered, "{plan:#?}");
    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    let comment = plan.comment.as_deref().expect("comment");
    assert!(comment.contains("dependency-baseline-missing"));
    assert!(comment.contains("`Package.swift`"));
    assert!(comment.contains("`Package.resolved`"));
}
