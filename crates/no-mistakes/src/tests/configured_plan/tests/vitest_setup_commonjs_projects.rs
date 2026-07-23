use super::*;

#[test]
fn commonjs_project_arrays_keep_resolved_setup_owners_exact() {
    let (_fixture, root) = vitest_setup_fixture();
    for (setup, test) in [
        (
            "cjs-direct-element-owner/setup/direct-element.ts",
            "cjs-direct-element-owner/cjs-direct-element.test.ts",
        ),
        (
            "cjs-direct-spread-owner/setup/direct-spread.ts",
            "cjs-direct-spread-owner/cjs-direct-spread.test.ts",
        ),
        (
            "cjs-named-member-owner/setup/named-member.ts",
            "cjs-named-member-owner/cjs-named-member.test.ts",
        ),
        (
            "cjs-named-object-owner/setup/named-object.ts",
            "cjs-named-object-owner/cjs-named-object.test.ts",
        ),
        (
            "cjs-named-replacement-owner/setup/named-replacement.ts",
            "cjs-named-replacement-owner/cjs-named-replacement.test.ts",
        ),
    ] {
        let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        assert_eq!(plan.selected_tests.len(), 1, "{setup}: {plan:#?}");
        assert_eq!(plan.selected_tests[0].test_file, test, "{setup}: {plan:#?}");
        assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
    }
}

#[test]
fn commonjs_project_exclusions_and_default_members_have_no_owner() {
    let (_fixture, root) = vitest_setup_fixture();
    for setup in [
        "cjs-default-member-owner/setup/default-member.ts",
        "cjs-require-excluded-owner/setup/require-excluded.ts",
        "cjs-named-excluded-owner/setup/named-excluded.ts",
    ] {
        let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
            root.clone(),
            vec![root.join(setup)],
        ))
        .unwrap();
        assert!(plan.selected_tests.is_empty(), "{setup}: {plan:#?}");
        assert!(!plan.fallback_triggered, "{setup}: {plan:#?}");
    }
}
