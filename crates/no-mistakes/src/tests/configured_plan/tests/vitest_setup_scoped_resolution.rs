use super::*;

fn fixture(name: &str) -> (tempfile::TempDir, PathBuf) {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan")
            .join(name),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

#[test]
fn project_scoped_tsconfig_resolves_setup_alias_after_catalog_finalization() {
    let (_fixture, root) = fixture("vitest-project-tsconfig-setup");
    let mut args = vitest_setup_args(
        root.clone(),
        vec![root.join("packages/unit/setup/aliased.ts")],
    );
    args.config = Some(root.join(".no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["packages/unit/tests/owner.test.ts"]
    );
    assert!(!plan.fallback_triggered, "{plan:#?}");
}

#[test]
fn deleted_runtime_setup_candidate_falls_back_when_only_declaration_remains() {
    let (_fixture, root) = fixture("vitest-declaration-runtime-deleted");
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.config = Some(root.join(".no-mistakes.yml"));
    args.diff_content = Some(
        "diff --git a/setup/runtime.ts b/setup/runtime.ts\n\
--- a/setup/runtime.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const runtimeSetup = true\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["tests/owner.test.ts"]
    );
}
