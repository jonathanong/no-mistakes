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
    graph_rule_findings(
        root,
        config,
        None,
        &shared,
        None,
        Some(&graph),
        None,
        &crate::codebase::analysis_session::PathInterner::new(),
    )
    .unwrap_err()
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

#[test]
fn propagates_required_entrypoint_reachability_errors() {
    let config = invalid_filter_config(
        REQUIRED_ENTRYPOINT_REACHABILITY,
        "sourceGlobs: [sources/static.ts]\nentrypoints: [entrypoints/api.ts]",
    );
    let error = error_from(&config);
    assert!(error.to_string().contains("include contains invalid glob"));
}
