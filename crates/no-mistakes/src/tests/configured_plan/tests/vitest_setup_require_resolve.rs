use super::*;

#[test]
fn dynamic_setup_require_resolve_loaders_keep_owner_fallback_for_edits_and_deletions() {
    let (_fixture, root) = vitest_setup_fixture();
    for args in [
        vitest_setup_args(
            root.clone(),
            vec![root.join("config/dynamic-resolved-loader.ts")],
        ),
        PlanArgs {
            diff_content: Some(
                "diff --git a/config/dynamic-resolved-loader.ts b/config/dynamic-resolved-loader.ts\n\
--- a/config/dynamic-resolved-loader.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const dynamicResolvedLoader = true\n"
                    .to_string(),
            ),
            ..vitest_setup_args(root.clone(), Vec::new())
        },
    ] {
        let plan = crate::tests::plan::generate_plan(&args).unwrap();
        assert_eq!(
            plan.selected_tests
                .iter()
                .map(|test| test.test_file.as_str())
                .collect::<Vec<_>>(),
            ["commonjs-closure-owner/commonjs-closure.test.ts"],
            "{plan:#?}"
        );
        assert!(plan.fallback_triggered, "{plan:#?}");
    }

    let mut nonliteral = vitest_setup_args(root, Vec::new());
    nonliteral.diff_content = Some(
        "diff --git a/config/nonliteral-dynamic-loader.ts b/config/nonliteral-dynamic-loader.ts\n\
--- a/config/nonliteral-dynamic-loader.ts\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export const nonliteral = true\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&nonliteral).unwrap();
    assert!(plan.selected_tests.is_empty(), "{plan:#?}");
    assert!(!plan.fallback_triggered, "{plan:#?}");
}
