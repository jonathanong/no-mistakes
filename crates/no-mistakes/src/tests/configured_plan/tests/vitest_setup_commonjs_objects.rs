use super::*;

#[test]
fn commonjs_destructured_and_object_exports_keep_exact_setup_owners() {
    let (_fixture, root) = vitest_setup_fixture();
    for (setup, test) in [
        (
            "shared-setup/destructured-commonjs.ts",
            "destructured-commonjs-require/destructured-commonjs-require.test.ts",
        ),
        (
            "shared-setup/module-exports-object.ts",
            "module-exports-object/module-exports-object.test.ts",
        ),
    ] {
        let changed = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
        deleted_args.diff_content = Some(format!(
            "diff --git a/{setup} b/{setup}\n--- a/{setup}\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const setup = true\n"
        ));
        let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

        for plan in [changed, deleted] {
            assert_eq!(
                plan.selected_tests
                    .iter()
                    .map(|test| test.test_file.as_str())
                    .collect::<Vec<_>>(),
                [test],
                "{setup}: {plan:#?}"
            );
            assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
        }
    }
}

#[test]
fn deleted_commonjs_object_helper_keeps_only_its_setup_owners() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root, Vec::new());
    args.diff_content = Some(
        "diff --git a/config/module-exports-object-setups.cjs b/config/module-exports-object-setups.cjs\n--- a/config/module-exports-object-setups.cjs\n+++ /dev/null\n@@ -1 +0,0 @@\n-module.exports = {}\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        [
            "destructured-commonjs-require/destructured-commonjs-require.test.ts",
            "module-exports-object/module-exports-object.test.ts",
        ],
        "{plan:#?}"
    );
    // A deleted helper conservatively falls back, but only to its two owners.
    assert!(plan.fallback_triggered, "{plan:#?}");
}

#[test]
fn commonjs_module_replacements_do_not_restore_stale_named_setup_edges() {
    let (_fixture, root) = vitest_setup_fixture();
    for setup in [
        "shared-setup/replaced-object-old.ts",
        "shared-setup/replaced-object-detached.ts",
        "shared-setup/replaced-nonobject-old.ts",
        "shared-setup/alias-barrier-detached.ts",
        "shared-setup/module-override-original.ts",
    ] {
        let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        assert!(plan.selected_tests.is_empty(), "{setup}: {plan:#?}");
    }

    for setup in [
        "shared-setup/alias-barrier-retained.ts",
        "shared-setup/module-override.ts",
    ] {
        let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        assert_eq!(
            plan.selected_tests[0].test_file,
            "commonjs-replacement-owner/commonjs-replacement.test.ts",
            "{setup}: {plan:#?}"
        );
        assert_eq!(plan.selected_tests.len(), 1, "{setup}: {plan:#?}");
        assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
    }
}

#[test]
fn standalone_imported_setup_config_changes_select_its_exact_owner() {
    let (_fixture, root) = vitest_setup_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("vitest.standalone-imported-project.ts")],
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests[0].test_file,
        "standalone-imported-setup-owner/standalone-imported-setup.test.ts",
        "{plan:#?}"
    );
    assert_eq!(plan.selected_tests.len(), 1, "{plan:#?}");
    // Config edits conservatively fall back, but only to this setup owner.
    assert!(plan.fallback_triggered, "{plan:#?}");
}
