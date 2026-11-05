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
        sources: None,
        tsconfig_catalog: None,
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
        sources: None,
        tsconfig_catalog: None,
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

#[test]
fn failed_application_retains_healthy_findings_for_the_same_rule() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-consumer-isolation"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("applications.yml"))).unwrap();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            imports: true,
            calls: true,
            mock_boundaries: true,
            ..Default::default()
        },
    )
    .unwrap();
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    let output = graph_rule_findings_with_session(
        GraphRuleRequest {
            root: &root,
            config: &config,
            config_path: None,
            shared: &shared,
            prepared_graph: None,
            dependency_graph: Some(&graph),
            inferred_roots: None,
            sources: None,
            tsconfig_catalog: None,
            prepared_vitest_projects: None,
            prepared_playwright_projects: None,
        },
        Some(&session),
    )
    .unwrap();
    assert_eq!(output.findings.len(), 1, "{:?}", output.findings);
    assert!(output.findings[0].file.ends_with("healthy.ts"));
    assert!(output.findings[0].message.contains("application #2"));
    assert_eq!(session.check_failures().len(), 1);
    assert!(session.check_failures()[0].message.contains("broken.ts"));
}

fn graph_rule_findings(
    request: GraphRuleRequest<'_>,
) -> Result<crate::codebase::rules::analysis_policy::AnalysisDiagnostics> {
    graph_rule_findings_with_session(request, None)
}

#[test]
fn failed_consumers_do_not_dispatch_without_their_canonical_graph() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation"),
    );
    for consumer in [
        "production-exports-only-used-by-tests",
        FORBIDDEN_CALLS,
        "test-no-live-repository-scan",
        FORBIDDEN_DEPENDENCIES,
        REQUIRED_ENTRYPOINT_REACHABILITY,
        UNCONSTRUCTED_ERROR_CLASS,
        "query-reached-per-item",
        "postgres-no-mutable-sql-registry",
        "postgres-no-unbounded-sql-identifier",
    ] {
        let config = invalid_filter_config(consumer, "{}");
        let shared = crate::codebase::check_facts::CheckFactMap::default();
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        session.enable_partial_checks();
        session.record_check_failure(crate::codebase::check_outcome::CheckFailure {
            consumer: consumer.into(),
            file: Some(root.join("broken.ts")),
            property: None,
            message: "canonical graph input could not be parsed".into(),
        });
        let output = graph_rule_findings_with_session(
            GraphRuleRequest {
                root: &root,
                config: &config,
                config_path: None,
                shared: &shared,
                prepared_graph: None,
                dependency_graph: None,
                inferred_roots: None,
                sources: None,
                tsconfig_catalog: None,
                prepared_vitest_projects: None,
                prepared_playwright_projects: None,
            },
            Some(&session),
        )
        .unwrap();
        assert!(output.findings.is_empty(), "{consumer}");
        assert!(output.advisories.is_empty(), "{consumer}");
        let failures = session.check_failures();
        assert_eq!(failures.len(), 1, "{consumer}");
        assert_eq!(
            failures[0].file.as_deref(),
            Some(root.join("broken.ts").as_path())
        );
    }
}

#[test]
fn ordinary_graph_dispatch_propagates_owned_configuration_errors() {
    for (consumer, options, expected) in [
        (
            FORBIDDEN_CALLS,
            "{}",
            "requires at least one options.roots selector",
        ),
        (
            "test-no-live-repository-scan",
            "fixtureDirs: ['a/b']",
            "fixtureDirs must list path segments",
        ),
        (
            UNCONSTRUCTED_ERROR_CLASS,
            "{}",
            "include contains invalid glob",
        ),
        (
            "production-exports-only-used-by-tests",
            "productionRoots: [src]\ntestGlobs: ['**/*.test.ts']",
            "include contains invalid glob",
        ),
        (
            "query-reached-per-item",
            "{}",
            "options.effects must select at least one",
        ),
    ] {
        let error = error_from(&invalid_filter_config(consumer, options));
        assert!(
            error.to_string().contains(expected),
            "{consumer}: {error:#}"
        );
    }
}

#[test]
fn source_demand_failure_is_partial_or_strict_for_the_same_saved_source() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation"),
    );
    let broken = root.join("broken.ts");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![broken.clone()],
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            ..Default::default()
        },
    );
    assert!(shared.ts[&broken].parse_error.is_some());
    let mut config = invalid_filter_config(NEXTJS_NO_CACHING, "{}");
    config.rules[0].include = vec!["broken.ts".into()];
    for partial in [false, true] {
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        if partial {
            session.enable_partial_checks();
        }
        let result = super::super::outcomes::register_demands(&root, &config, &shared, &session);
        assert_eq!(result.is_ok(), partial);
        let failures = session.check_failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].consumer, NEXTJS_NO_CACHING);
        assert_eq!(failures[0].file.as_deref(), Some(broken.as_path()));
        assert_eq!(
            Some(failures[0].message.as_str()),
            shared.ts[&broken].parse_error.as_deref()
        );
    }
}

#[test]
fn invalid_demand_filter_is_recorded_before_partial_source_registration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = invalid_filter_config(NEXTJS_NO_CACHING, "{}");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    for partial in [false, true] {
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        if partial {
            session.enable_partial_checks();
        }
        let result = super::super::outcomes::register_demands(root, &config, &shared, &session);
        assert_eq!(result.is_ok(), partial);
        if partial {
            let failures = session.check_failures();
            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].consumer, NEXTJS_NO_CACHING);
            assert!(failures[0]
                .message
                .contains("include contains invalid glob"));
        } else {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("include contains invalid glob"));
        }
    }
}
