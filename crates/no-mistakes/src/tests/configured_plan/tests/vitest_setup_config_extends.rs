use super::*;

fn config_extends_fixture() -> (tempfile::TempDir, PathBuf) {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-extends-config"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    (fixture, root)
}

#[test]
fn static_vitest_config_extends_keeps_setup_ownership_exact() {
    let (_fixture, root) = config_extends_fixture();
    let plan = crate::tests::plan::generate_plan(&vitest_setup_args(
        root.clone(),
        vec![root.join("base-setup.ts")],
    ))
    .unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["extended/owned.test.ts"],
        "{plan:#?}"
    );
    assert!(!plan.fallback_triggered, "{plan:#?}");
    assert_eq!(plan.selected_tests[0].reasons[0].via, ["vitest-setup"]);
}

#[test]
fn unresolved_static_vitest_config_extends_uses_owner_fallback() {
    let (_fixture, root) = config_extends_fixture();
    let mut args = vitest_setup_args(root, Vec::new());
    args.diff_content = Some(
        "diff --git a/missing-vite.config.js b/missing-vite.config.js\n\
--- a/missing-vite.config.js\n\
+++ /dev/null\n\
@@ -1 +0,0 @@\n\
-export default {}\n"
            .to_string(),
    );
    let plan = crate::tests::plan::generate_plan(&args).unwrap();

    assert_eq!(
        plan.selected_tests
            .iter()
            .map(|test| test.test_file.as_str())
            .collect::<Vec<_>>(),
        ["unresolved/owned.test.ts"],
        "{plan:#?}"
    );
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.r#type == "vitest-config-extends-unresolved"));
}
