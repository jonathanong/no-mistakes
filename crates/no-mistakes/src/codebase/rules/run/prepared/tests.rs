use super::*;

#[test]
fn legacy_prepared_request_without_sources_uses_the_request_session() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let config = crate::config::v2::NoMistakesConfig::default();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);

    let findings = run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
        session: crate::codebase::analysis_session::AnalysisSession::disabled(),
        root: &root,
        config_path: None,
        tsconfig_path: None,
        shared: &shared,
        prepared_playwright: None,
        config: &config,
        prepared_graph: None,
        prepared_tsconfig: &tsconfig,
        prepared_tsconfig_catalog: &catalog,
        inferred_roots: None,
        sources: None,
    })
    .unwrap();

    assert!(findings.is_empty());
}
