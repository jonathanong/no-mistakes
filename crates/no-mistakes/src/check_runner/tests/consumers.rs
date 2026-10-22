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
    assert!(!needs_tsconfig_catalog(&config));
    assert!(!needs_workspace(&config, None));
    assert!(!needs_workspace(&config, Some(tsconfig())));
    assert!(!needs_inferred_roots(&config));
}

#[test]
fn framework_projects_infer_roots_without_a_catalog() {
    for type_ in ["nextjs", "remix", "vitejs"] {
        let config = config(&format!("projects:\n  app:\n    type: {type_}\n"));
        assert!(
            !needs_tsconfig_catalog(&config),
            "{type_} must not pay for a tsconfig catalog"
        );
        assert!(
            needs_inferred_roots(&config),
            "{type_} still needs inferred framework roots"
        );
        assert!(!needs_workspace(&config, None));
    }
}

#[test]
fn server_projects_do_not_infer_framework_roots() {
    let config = config("projects:\n  api:\n    type: server\n");
    assert!(!needs_inferred_roots(&config));
}

#[test]
fn unique_exports_needs_catalog_but_skips_workspace_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: unique-exports\n    scope: repository\n");
    assert!(needs_tsconfig_catalog(&config));
    assert!(needs_workspace(&config, None));
    assert!(!needs_workspace(&config, Some(tsconfig())));
    assert!(needs_inferred_roots(&config));
}

#[test]
fn tsconfig_gate_coverage_needs_workspace_even_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: tsconfig-gate-coverage\n    scope: repository\n");
    assert!(needs_workspace(&config, Some(tsconfig())));
}

#[test]
fn playwright_coverage_needs_workspace_even_with_explicit_tsconfig() {
    let config = config("rules:\n  - rule: playwright-coverage\n    scope: repository\n");
    assert!(needs_workspace(&config, Some(tsconfig())));
}

#[test]
fn react_assert_no_fetch_needs_catalog() {
    let config = config("reactTraits:\n  frontendRoot: app\n  assertNoFetch: true\n");
    assert!(needs_tsconfig_catalog(&config));
}

#[test]
fn queues_need_catalog() {
    let config = config(
        "projects:\n  api:\n    type: server\n    queues:\n      enqueues:\n        - enqueues/*.ts\n",
    );
    assert!(needs_tsconfig_catalog(&config));
}
