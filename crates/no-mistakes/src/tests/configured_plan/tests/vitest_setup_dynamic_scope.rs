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

fn selected_files(plan: &crate::tests::TestPlan) -> Vec<String> {
    let mut files = plan
        .selected_tests
        .iter()
        .map(|test| test.test_file.clone())
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn saved_config_says_the_env_read_is_dynamic(name: &str) {
    let source = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan")
            .join(name)
            .join("vitest.config.ts"),
    )
    .unwrap();
    assert!(
        source.contains("Dynamic on purpose: the analyzer cannot resolve this statically."),
        "{source}"
    );
}

fn plan_for(name: &str, changed: &str) -> crate::tests::TestPlan {
    let (_fixture, root) = fixture(name);
    crate::tests::plan::generate_plan(&vitest_setup_args(root.clone(), vec![root.join(changed)]))
        .unwrap()
}

fn warning_names_setup_declaration(plan: &crate::tests::TestPlan, project: &str) {
    assert!(
        plan.warnings.iter().any(|warning| {
            warning.r#type == "vitest-setup-dynamic"
                && warning.file == "vitest.config.ts"
                && warning.message.contains("setupFiles")
                && warning.message.contains(project)
                && warning.message.contains("vitest.config.ts")
        }),
        "{plan:#?}"
    );
}

#[test]
fn known_owner_dynamic_setup_files_select_only_that_owner() {
    saved_config_says_the_env_read_is_dynamic("vitest-dynamic-setup-scope/known-owner");
    let plan = plan_for("vitest-dynamic-setup-scope/known-owner", "owner/source.ts");
    assert_eq!(selected_files(&plan), ["owner/owner.test.ts"]);
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert!(plan.fallback_reason.as_deref().is_some_and(|reason| {
        reason.contains("owning project") && !reason.contains("discovered Vitest tests")
    }));
    warning_names_setup_declaration(&plan, "owner");
}

#[test]
fn unknown_owner_dynamic_setup_files_select_the_vitest_framework_set() {
    // A parsed root project keeps its config path and scope, so the reason
    // stays owner-scoped. That owner's include is the discovered Vitest set.
    saved_config_says_the_env_read_is_dynamic("vitest-dynamic-setup-scope/unknown-owner");
    let plan = plan_for("vitest-dynamic-setup-scope/unknown-owner", "unit/source.ts");
    assert_eq!(selected_files(&plan), ["unit/a.test.ts", "unit/b.test.ts"]);
    assert!(plan.fallback_triggered, "{plan:#?}");
    assert_eq!(
        plan.fallback_reason.as_deref(),
        Some(
            "Vitest setup dependencies could not be resolved statically; selected owning project tests"
        ),
        "{plan:#?}"
    );
    warning_names_setup_declaration(&plan, "default");
}
