use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
use crate::tests::{PlanArgs, PlanIncompletenessKind, TestFramework};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn upgraded_fixture() -> (tempfile::TempDir, PathBuf) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dependency-baseline-incomplete/fixture");
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    git_init(&root);
    git_commit_all(&root, "base");
    for (from, to) in [
        ("changes/package.json", "package.json"),
        ("changes/tooling/package.json", "tooling/package.json"),
        ("changes/pnpm-lock.yaml", "pnpm-lock.yaml"),
    ] {
        std::fs::copy(root.join(from), root.join(to)).unwrap();
    }
    (fixture, root)
}

fn plan(root: &Path, base: Option<&str>) -> crate::tests::TestPlan {
    crate::tests::plan::generate_plan(&PlanArgs {
        framework: Some(TestFramework::Vitest),
        root: root.to_path_buf(),
        config: None,
        tsconfig: None,
        base: base.map(str::to_string),
        head: None,
        from_git_diff: None,
        changed_file: vec![
            root.join("package.json"),
            root.join("tooling/package.json"),
            root.join("pnpm-lock.yaml"),
        ],
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
        global_config_fallback: None,
        direct_test_owner: false,
        format: None,
        json: false,
        include_comment: false,
        include_glob: Vec::new(),
    })
    .unwrap()
}

fn selected_files(plan: &crate::tests::TestPlan) -> BTreeSet<&str> {
    plan.selected_tests
        .iter()
        .map(|test| test.test_file.as_str())
        .collect()
}

#[test]
fn missing_dependency_baseline_is_incomplete_and_does_not_select_the_suite() {
    let (_fixture, root) = upgraded_fixture();
    let plan = plan(&root, None);
    let json = serde_json::to_value(&plan).unwrap();

    assert_eq!(
        json["incomplete"],
        serde_json::json!([{
            "kind": PlanIncompletenessKind::DependencyBaselineMissing.as_str(),
            "files": ["package.json", "pnpm-lock.yaml", "tooling/package.json"]
        }]),
        "{plan:#?}"
    );
    assert!(
        plan.warnings
            .iter()
            .any(|warning| warning.r#type == "lockfile-no-baseline"
                && warning.file == "pnpm-lock.yaml"),
        "{plan:#?}"
    );
    assert_eq!(
        plan.warnings
            .iter()
            .filter(|warning| warning.r#type == "package-manifest-no-baseline")
            .map(|warning| warning.file.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["package.json", "tooling/package.json"]),
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(
        plan.fallback_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("selected owning project tests")),
        "{plan:#?}"
    );
    assert_eq!(
        selected_files(&plan),
        BTreeSet::from(["app/app.test.ts"]),
        "{plan:#?}"
    );
    assert!(
        !plan
            .execution_targets
            .iter()
            .any(|target| target.project.as_deref() == Some("tooling")),
        "{plan:#?}"
    );
}

#[test]
fn dependency_baseline_selects_direct_and_transitive_tooling_importers() {
    let (_fixture, root) = upgraded_fixture();
    let plan = plan(&root, Some("HEAD"));
    let json = serde_json::to_value(&plan).unwrap();

    assert!(json.get("incomplete").is_none(), "{plan:#?}");
    assert!(
        !plan.warnings.iter().any(|warning| {
            warning.r#type == "lockfile-no-baseline"
                || warning.r#type == "package-manifest-no-baseline"
        }),
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        selected_files(&plan),
        BTreeSet::from([
            "app/app.test.ts",
            "tooling/direct.test.ts",
            "tooling/indirect.test.ts",
        ]),
        "{plan:#?}"
    );

    let tooling = plan
        .execution_targets
        .iter()
        .find(|target| target.project.as_deref() == Some("tooling"))
        .expect("tooling execution target");
    assert_eq!(
        tooling.test_files,
        vec![
            "tooling/direct.test.ts".to_string(),
            "tooling/indirect.test.ts".to_string(),
        ],
        "{plan:#?}"
    );

    let direct = plan
        .selected_tests
        .iter()
        .find(|test| test.test_file == "tooling/direct.test.ts")
        .unwrap();
    assert!(
        direct.reasons.iter().any(|reason| {
            reason.path.windows(2).any(|step| {
                step[0].contains("fixture-analyzer") && step[1] == "tooling/direct.test.ts"
            })
        }),
        "{direct:#?}"
    );
    let indirect = plan
        .selected_tests
        .iter()
        .find(|test| test.test_file == "tooling/indirect.test.ts")
        .unwrap();
    assert!(
        indirect.reasons.iter().any(|reason| {
            reason.path.windows(3).any(|step| {
                step[0].contains("fixture-analyzer")
                    && step[1] == "tooling/helper.ts"
                    && step[2] == "tooling/indirect.test.ts"
            })
        }),
        "{indirect:#?}"
    );
}
