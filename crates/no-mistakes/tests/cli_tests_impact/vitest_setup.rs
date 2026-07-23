use super::*;

fn vitest_setup_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    )
}

fn impact(root: &std::path::Path, entrypoint: &str) -> serde_json::Value {
    let output = run(&[
        "tests",
        "impact",
        entrypoint,
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(&output)).unwrap()
}

#[test]
fn tests_impact_keeps_unresolved_literal_vitest_setup_as_an_entrypoint() {
    let plan = impact(&vitest_setup_root(), "inherits/setup/missing.ts");
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 1, "{plan:#}");
    assert_eq!(selected[0]["test_file"], "inherits/inherited.test.ts");
    assert!(
        selected[0]["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| {
                reason["via"]
                    .as_array()
                    .is_some_and(|via| via.last().is_some_and(|edge| edge == "vitest-setup"))
            }),
        "{plan:#}"
    );
}

#[test]
fn tests_impact_keeps_explicit_non_code_vitest_project_matches() {
    let plan = impact(
        &vitest_setup_root(),
        "arbitrary-project-match/setup/arbitrary.ts",
    );
    assert_eq!(
        plan["selected_tests"][0]["test_file"], "arbitrary-project-match/arbitrary.fixture",
        "{plan:#}"
    );
}

#[test]
fn tests_impact_applies_owner_fallback_for_dynamic_vitest_setup() {
    let plan = impact(&vitest_setup_root(), "config/setup-selector.ts");
    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "inherits/inherited.test.ts"
    );
    assert!(
        plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| { warning["type"] == "vitest-setup-dynamic" }),
        "{plan:#}"
    );
}

#[test]
fn tests_impact_applies_owner_fallback_for_deleted_setup_helper() {
    let plan = impact(
        &vitest_setup_root(),
        "runtime-owner/setup/deleted-runtime-helper.ts",
    );
    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(
        plan["selected_tests"][0]["test_file"], "runtime-owner/runtime-owner.test.ts",
        "{plan:#}"
    );
    assert!(
        plan["fallback_reason"].as_str().is_some_and(
            |reason| reason.contains("transitive dependency of a resolved setup was deleted")
        ),
        "{plan:#}"
    );
}

#[test]
fn tests_impact_discovers_configless_vitest_project_folders() {
    let root = vitest_setup_root();
    let plan = plan_for(&root, "configless-project/default.test.ts");
    assert_eq!(
        plan["selected_tests"][0]["test_file"], "configless-project/default.test.ts",
        "{plan:#}"
    );
}

#[test]
fn tests_plan_setup_fallback_spends_dependency_group_budget() {
    let root = vitest_setup_root();
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--config",
        "dependency-limit.no-mistakes.yml",
        "--changed-file",
        "setup/conditional-a.ts",
        "--changed-file",
        "config/setup-selector.ts",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true, "{plan:#}");
    assert_eq!(plan["groups"].as_array().unwrap().len(), 1, "{plan:#}");
    assert_eq!(plan["groups"][0]["type"], "dependencies");
    assert_eq!(plan["groups"][0]["limit"], 1);
    assert_eq!(plan["groups"][0]["selected"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
}
