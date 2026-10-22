use super::prepared::consumers::{needs_inferred_roots, needs_tsconfig_catalog, needs_workspace};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;

fn config(yaml: &str) -> NoMistakesConfig {
    serde_yaml::from_str(yaml).unwrap()
}

fn tsconfig() -> &'static Path {
    Path::new("tsconfig.json")
}

#[test]
fn empty_config_skips_catalog_workspace_and_inferred_roots() {
    let config = config("{}");
    assert!(!needs_tsconfig_catalog(&config, false));
    assert!(!needs_workspace(&config, None, false));
    assert!(!needs_workspace(&config, Some(tsconfig()), false));
    assert!(!needs_inferred_roots(&config, false));
}

#[test]
fn framework_projects_infer_roots_without_a_catalog() {
    for type_ in ["nextjs", "remix", "vitejs"] {
        let config = config(&format!("projects:\n  app:\n    type: {type_}\n"));
        assert!(
            !needs_tsconfig_catalog(&config, false),
            "{type_} must not pay for a tsconfig catalog"
        );
        assert!(
            needs_inferred_roots(&config, false),
            "{type_} still needs inferred framework roots"
        );
        assert!(!needs_workspace(&config, None, false));
    }
}

#[test]
fn server_projects_do_not_infer_framework_roots() {
    let config = config("projects:\n  api:\n    type: server\n");
    assert!(!needs_inferred_roots(&config, false));
}

#[test]
fn unique_exports_needs_catalog_but_skips_workspace_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: unique-exports\n    scope: repository\n");
    assert!(needs_tsconfig_catalog(&config, false));
    assert!(needs_workspace(&config, None, false));
    assert!(!needs_workspace(&config, Some(tsconfig()), false));
    assert!(needs_inferred_roots(&config, false));
}

#[test]
fn tsconfig_gate_coverage_needs_workspace_even_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: tsconfig-gate-coverage\n    scope: repository\n");
    assert!(needs_workspace(&config, Some(tsconfig()), false));
}

#[test]
fn playwright_coverage_needs_workspace_even_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: playwright-coverage\n    scope: repository\n");
    assert!(needs_workspace(&config, Some(tsconfig()), false));
}

#[test]
fn react_assert_no_fetch_needs_catalog() {
    let config = config("reactTraits:\n  frontendRoot: app\n  assertNoFetch: true\n");
    assert!(needs_tsconfig_catalog(&config, false));
}

#[test]
fn queues_need_catalog() {
    let config = config(
        "projects:\n  api:\n    type: server\n    queues:\n      enqueues:\n        - enqueues/*.ts\n",
    );
    assert!(needs_tsconfig_catalog(&config, false));
}

#[test]
fn prepared_embedded_sql_rules_need_catalog() {
    let config = config("rules:\n  - rule: postgres-idempotent-insert\n    scope: repository\n");
    assert!(needs_tsconfig_catalog(&config, false));
    assert!(needs_workspace(&config, None, false));
}

#[test]
fn declared_runner_configs_need_catalog_only_for_deadline_evidence() {
    let config = config("tests:\n  vitest:\n    configs: vitest.config.ts\n");
    assert!(!needs_tsconfig_catalog(&config, false));
    assert!(needs_tsconfig_catalog(&config, true));
    assert!(!needs_workspace(&config, None, false));
    assert!(needs_workspace(&config, None, true));
    assert!(!needs_workspace(&config, Some(tsconfig()), true));
}

#[test]
fn deadline_evidence_without_declared_runner_configs_skips_catalog() {
    let config = config("{}");
    assert!(!needs_tsconfig_catalog(&config, true));
    assert!(!needs_workspace(&config, None, true));
}
