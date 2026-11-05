use super::*;

#[test]
fn missing_aggregate_playwright_prerequisite_is_incomplete_without_standalone_repair() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-runner/empty"),
    );
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: crate::playwright::rules::PLAYWRIGHT_COVERAGE.into(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);
    let sources = snapshot.source_store_for(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig::default();
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let output = playwright(&IndependentRuleRequest {
        session: &session,
        root: &root,
        config_path: None,
        shared: &facts,
        prepared_playwright: None,
        config: &config,
        prepared_graph: None,
        prepared_tsconfig_catalog: &catalog,
        prepared_vitest_projects: None,
        prepared_playwright_projects: None,
        inferred_roots: None,
        sources: &sources,
        dependency_graph: None,
        defer_suppression: false,
    })
    .unwrap();
    assert!(output.findings.is_empty());
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0].consumer,
        crate::playwright::rules::PLAYWRIGHT_COVERAGE
    );
}
