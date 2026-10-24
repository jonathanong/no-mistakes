use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use std::collections::HashMap;

fn invalid_filter_config(rule: &str, options: &str) -> crate::config::v2::NoMistakesConfig {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: rule.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["[".to_string()],
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

fn error_from(config: &crate::config::v2::NoMistakesConfig) -> anyhow::Error {
    let root = Path::new("/repo");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.to_path_buf(),
        HashMap::new(),
        HashMap::new(),
    );
    graph_rule_findings(GraphRuleRequest {
        root,
        config,
        config_path: None,
        shared: &shared,
        prepared_graph: None,
        dependency_graph: Some(&graph),
        inferred_roots: None,
        prepared_vitest_projects: None,
        prepared_playwright_projects: None,
    })
    .err()
    .expect("invalid graph rule must fail")
}

#[test]
fn propagates_forbidden_dependencies_fact_errors() {
    let config = invalid_filter_config(
        FORBIDDEN_DEPENDENCIES,
        "roots: [entrypoints/api.mts]\nforbiddenModules: [sharp]",
    );
    let error = error_from(&config);
    assert!(
        error
            .to_string()
            .contains("shared check facts are missing graph facts"),
        "unexpected error: {error:#}"
    );
}

fn panic_without_graph(rule: &str) {
    let config = invalid_filter_config(rule, "importSpecifier: \"@example/db\"");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let _ = graph_rule_findings(GraphRuleRequest {
        root: Path::new("/repo"),
        config: &config,
        config_path: None,
        shared: &shared,
        prepared_graph: None,
        dependency_graph: None,
        inferred_roots: None,
        prepared_vitest_projects: None,
        prepared_playwright_projects: None,
    });
}

#[test]
#[should_panic(expected = "postgres-no-mutable-sql-registry requires canonical graph")]
fn mutable_registry_requires_canonical_graph() {
    panic_without_graph("postgres-no-mutable-sql-registry");
}

#[test]
#[should_panic(expected = "postgres-no-unbounded-sql-identifier requires canonical graph")]
fn unbounded_identifier_requires_canonical_graph() {
    panic_without_graph("postgres-no-unbounded-sql-identifier");
}

#[test]
fn propagates_sql_cross_file_filter_errors() {
    for rule in [
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ] {
        let error = error_from(&invalid_filter_config(
            rule,
            "importSpecifier: \"@example/db\"",
        ));
        assert!(
            error.to_string().contains("include contains invalid glob"),
            "{rule}: {error:#}"
        );
    }
}

#[test]
fn propagates_required_entrypoint_reachability_errors() {
    let config = invalid_filter_config(
        REQUIRED_ENTRYPOINT_REACHABILITY,
        "sourceGlobs: [sources/static.ts]\nentrypoints: [entrypoints/api.ts]",
    );
    let error = error_from(&config);
    assert!(error.to_string().contains("include contains invalid glob"));
}
