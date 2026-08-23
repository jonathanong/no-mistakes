use super::*;

#[test]
fn exhausted_setup_budget_tracks_changes_outside_the_owner() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-setup-bounds"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut changed_args = vitest_setup_args(
        root.clone(),
        vec![root.join("packages/foo/shared/outside.ts")],
    );
    changed_args.framework = None;
    let changed = crate::tests::plan::generate_plan(&changed_args).unwrap();
    let mut deleted_args = vitest_setup_args(root, Vec::new());
    deleted_args.framework = None;
    deleted_args.diff_content = Some(
        "diff --git a/packages/foo/shared/outside.ts b/packages/foo/shared/outside.ts\n--- a/packages/foo/shared/outside.ts\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const directOutsideSetup = true\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [changed, deleted] {
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["packages/foo/bounded.test.ts"]
        );
    }
}

#[test]
fn exhausted_imported_setup_literals_rebase_to_the_final_project_root() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-setup-bounds"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let changed = root.join("packages/shared/outside.ts");
    let changed_plan =
        crate::tests::plan::generate_plan(&vitest_setup_args(root.clone(), vec![changed])).unwrap();
    let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
    deleted_args.diff_content = Some(
        "diff --git a/packages/shared/outside.ts b/packages/shared/outside.ts\n--- a/packages/shared/outside.ts\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const rebasedOutsideSetup = true\n"
            .to_string(),
    );
    let deleted_plan = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [changed_plan, deleted_plan] {
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["packages/rebased/rebased.test.ts"],
            "{plan:#?}"
        );
    }
}

#[test]
fn resolved_setup_resource_changes_and_deletions_keep_the_owner_scoped() {
    let (_fixture, root) = vitest_setup_fixture();
    let changed = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("runtime-resource.json")],
    ))
    .unwrap();
    let mut deleted_args = vitest_setup_args(root, Vec::new());
    deleted_args.diff_content = Some(
        "diff --git a/runtime-resource.json b/runtime-resource.json\n--- a/runtime-resource.json\n+++ /dev/null\n@@ -1 +0,0 @@\n-{\"fixture\": true}\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [&changed, &deleted] {
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["runtime-owner/runtime-owner.test.ts"],
            "{plan:#?}"
        );
    }
    assert!(!changed.fallback_triggered, "{changed:#?}");
    assert!(deleted.fallback_triggered, "{deleted:#?}");
}
