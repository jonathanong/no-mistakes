use super::*;

#[test]
fn exhausted_setup_budget_tracks_changes_outside_the_owner() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-setup-bounds"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("shared/outside.ts")]);
    args.framework = None;
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["packages/foo/bounded.test.ts"]
    );
}

#[test]
fn union_plan_applies_dynamic_setup_warnings_and_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("config/setup-selector.ts")]);
    args.framework = None;
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "vitest-setup-dynamic"));
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"]
    );
}

#[test]
fn union_plan_applies_unresolved_setup_deletion_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.framework = None;
    args.diff_content = Some(
        "diff --git a/aliases/setup/missing.ts b/aliases/setup/missing.ts\n\
--- a/aliases/setup/missing.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const missing = true\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "vitest-setup-unresolved"));
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["alias-owner/alias-owner.test.ts"]
    );
}

#[test]
fn conditional_setup_branches_and_selector_keep_owner_provenance() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut branch_args =
        vitest_setup_args(root.clone(), vec![root.join("setup/conditional-a.ts")]);
    branch_args.framework = None;
    let branch = crate::tests::plan::generate_plan(&branch_args).unwrap();
    assert_eq!(
        branch
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["conditional-owner/conditional.test.ts"]
    );
    assert!(!branch.fallback_triggered, "{branch:#?}");

    let mut selector_args =
        vitest_setup_args(root.clone(), vec![root.join("config/branch-selector.ts")]);
    selector_args.framework = None;
    let selector = crate::tests::plan::generate_plan(&selector_args).unwrap();
    assert_eq!(
        selector
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["conditional-owner/conditional.test.ts"]
    );
    assert!(selector.fallback_triggered, "{selector:#?}");

    let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
    deleted_args.framework = None;
    deleted_args.diff_content = Some(
        "diff --git a/setup/conditional-b.ts b/setup/conditional-b.ts\n\
--- a/setup/conditional-b.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const conditionalB = true\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();
    assert_eq!(
        deleted
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["conditional-owner/conditional.test.ts"]
    );
    // Deleted setup targets remain authoritative phantom graph roots, so the
    // exact setup edge wins without requiring a conservative fallback.
    assert!(!deleted.fallback_triggered, "{deleted:#?}");
}

#[test]
fn commonjs_literal_setup_exports_create_owner_scoped_setup_edges() {
    let (_fixture, root) = vitest_setup_fixture();
    for setup in [
        "commonjs-values/setup/commonjs-default.ts",
        "commonjs-values/setup/commonjs-default-template.ts",
        "commonjs-values/setup/commonjs-named.ts",
        "commonjs-values/setup/commonjs-named-template.ts",
        "commonjs-require/setup/commonjs-required.ts",
    ] {
        let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            [if setup.starts_with("commonjs-require/") {
                "commonjs-require/commonjs-require.test.ts"
            } else {
                "commonjs-values/commonjs-values.test.ts"
            },],
            "{setup}: {plan:#?}"
        );
        assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
    }
}

#[test]
fn named_imported_setup_member_changes_and_deletions_keep_exact_owner_edges() {
    let (_fixture, root) = vitest_setup_fixture();
    for setup in [
        "named-member.ts",
        "named-member-source.ts",
        "named-member-imported.ts",
        "named-member-star.ts",
        "named-member-commonjs.ts",
    ] {
        let relative = format!("shared-setup/{setup}");
        let changed = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(&relative)],
        ))
        .unwrap();
        let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
        deleted_args.diff_content = Some(format!(
            "diff --git a/{relative} b/{relative}\n--- a/{relative}\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const removed = true\n"
        ));
        let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

        for plan in [changed, deleted] {
            assert_eq!(
                plan.selected_tests
                    .iter()
                    .map(|test| test.test_file.as_str())
                    .collect::<Vec<_>>(),
                ["named-member-owner/named-member.test.ts"],
                "{setup}: {plan:#?}"
            );
            assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
            assert!(plan.selected_tests[0]
                .reasons
                .iter()
                .any(|reason| { reason.via.last().is_some_and(|edge| edge == "vitest-setup") }));
        }
    }
}

#[test]
fn named_imported_setup_member_helper_provenance_keeps_its_owner() {
    let (_fixture, root) = vitest_setup_fixture();
    let changed = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("config/named-member-setups.ts")],
    ))
    .unwrap();
    let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
    deleted_args.diff_content = Some(
        "diff --git a/config/named-member-setups.ts b/config/named-member-setups.ts\n\
--- a/config/named-member-setups.ts\n\
+++ /dev/null\n\
@@ -1,3 +0,0 @@\n\
-export const namedMemberConfig = {\n\
-  files: '../shared-setup/named-member.ts',\n\
-}\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [changed, deleted] {
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["named-member-owner/named-member.test.ts"],
            "{plan:#?}"
        );
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.fallback_reason.as_deref(),
            Some("Vitest setup configuration changed; selected owning project tests")
        );
    }
}

#[test]
fn resolved_setup_runtime_loader_deletion_uses_its_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    for helper in ["required-helper", "dynamic-helper"] {
        let mut args = vitest_setup_args(root.clone(), Vec::new());
        args.diff_content = Some(format!(
            "diff --git a/runtime-owner/setup/{helper}.ts b/runtime-owner/setup/{helper}.ts\n--- a/runtime-owner/setup/{helper}.ts\n+++ /dev/null\n@@ -1 +0,0 @@\n-export const {helper} = true\n"
        ));
        let plan = crate::tests::plan::generate_plan(&args).unwrap();

        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["runtime-owner/runtime-owner.test.ts"],
            "{helper}: {plan:#?}"
        );
        assert!(plan.fallback_triggered, "{helper}: {plan:#?}");
        assert_eq!(
            plan.fallback_reason.as_deref(),
            Some("A transitive dependency of a resolved setup was deleted; selected owning project tests"),
            "{helper}: {plan:#?}"
        );
    }
}

#[test]
fn setup_fallback_respects_the_existing_dependency_group_budget() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(
        root.clone(),
        vec![
            root.join("setup/conditional-a.ts"),
            root.join("config/setup-selector.ts"),
        ],
    );
    args.config = Some(root.join("dependency-limit.no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(plan.groups.len(), 1, "{plan:#?}");
    assert_eq!(plan.groups[0].r#type, "dependencies");
    assert_eq!(plan.groups[0].limit, Some(1));
    assert_eq!(plan.groups[0].selected.len(), 1, "{plan:#?}");
    assert_eq!(plan.selected_tests.len(), 1, "{plan:#?}");
}

#[test]
fn setup_fallback_uses_later_dependency_group_capacity() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("config/setup-selector.ts")]);
    args.config = Some(root.join("dependency-multiple-groups.no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(plan.groups.len(), 2, "{plan:#?}");
    assert!(plan.groups[0].selected.is_empty(), "{plan:#?}");
    assert_eq!(
        plan.groups[1].selected,
        ["inherits/inherited.test.ts"],
        "{plan:#?}"
    );
    assert_eq!(plan.selected_tests.len(), 1, "{plan:#?}");
}

#[test]
fn resolved_setup_config_helpers_fall_back_but_setup_modules_keep_graph_edges() {
    let (_fixture, root) = vitest_setup_fixture();
    let config_helper = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("config/imported-setup-values.ts")],
    ))
    .unwrap();
    assert_eq!(
        config_helper
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["imported-values/imported-values.test.ts"],
        "{config_helper:#?}"
    );
    assert!(config_helper.fallback_triggered, "{config_helper:#?}");
    assert_eq!(
        config_helper.fallback_reason.as_deref(),
        Some("Vitest setup configuration changed; selected owning project tests")
    );

    let setup_module = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("imported-values/setup/imported-value.ts")],
    ))
    .unwrap();
    assert_eq!(
        setup_module
            .selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["imported-values/imported-values.test.ts"],
        "{setup_module:#?}"
    );
    assert!(!setup_module.fallback_triggered, "{setup_module:#?}");
}

#[test]
fn explicit_policy_override_keeps_dynamic_setup_fallback_in_parsed_owner_scope() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("inherits/dynamic-only.ts")]);
    args.config = Some(root.join("policy.no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["inherits/inherited.test.ts"],
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.fallback_reason.as_deref(),
        Some("Vitest setup dependencies could not be resolved statically; selected owning project tests")
    );
}

#[test]
fn explicit_policy_dynamic_setup_does_not_treat_the_repository_root_as_its_owner() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), vec![root.join("outside-policy-owner.ts")]);
    args.config = Some(root.join("policy.no-mistakes.yml"));
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    assert!(!plan.fallback_triggered, "{plan:#?}");
}

#[test]
fn deleted_resolved_setup_config_helper_uses_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.diff_content = Some(
        "diff --git a/config/imported-setup-values.ts b/config/imported-setup-values.ts\n\
--- a/config/imported-setup-values.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const importedSetupFiles = []\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["imported-values/imported-values.test.ts"],
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.fallback_reason.as_deref(),
        Some("Vitest setup configuration changed; selected owning project tests")
    );
}

#[test]
fn imported_setup_barrel_provenance_changes_and_deletions_use_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let changed = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("config/setup-barrel.ts")],
    ))
    .unwrap();
    let mut deleted_args = vitest_setup_args(root.clone(), Vec::new());
    deleted_args.diff_content = Some(
        "diff --git a/config/setup-barrel.ts b/config/setup-barrel.ts\n\
--- a/config/setup-barrel.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export * from './setup-barrel-leaf'\n"
            .to_string(),
    );
    let deleted = crate::tests::plan::generate_plan(&deleted_args).unwrap();

    for plan in [changed, deleted] {
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["imported-values/imported-values.test.ts"],
            "{plan:#?}"
        );
        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.fallback_reason.as_deref(),
            Some("Vitest setup configuration changed; selected owning project tests")
        );
    }
}

#[test]
fn declaration_only_setup_helper_is_not_a_runtime_fallback_trigger() {
    let (_fixture, root) = vitest_setup_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("config/declaration-only-setups.d.ts")],
    ))
    .unwrap();
    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    assert!(!plan.fallback_triggered, "{plan:#?}");
}

#[test]
fn deleted_missing_barrel_runtime_leaf_uses_its_owner_fallback() {
    let (_fixture, root) = vitest_setup_fixture();
    let mut args = vitest_setup_args(root.clone(), Vec::new());
    args.diff_content = Some(
        "diff --git a/config/missing-setup-barrel-leaf.ts b/config/missing-setup-barrel-leaf.ts\n\
--- a/config/missing-setup-barrel-leaf.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const missingBarrelSetups = []\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();
    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["missing-barrel/missing-barrel.test.ts"],
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
}
