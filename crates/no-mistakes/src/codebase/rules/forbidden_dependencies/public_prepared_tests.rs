use super::*;
use crate::codebase::dependencies::graph::GraphBuildPlan;

#[test]
fn public_prepared_check_rejects_a_broader_requested_plan() {
    let root = super::inventory_tests::saved_fixture("forbidden-dependencies-basic");
    let config_path = root.join("mixed-md.no-mistakes.yml");
    let requested = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let narrow = NoMistakesConfig::default();
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let codebase_config = crate::codebase::config::config_from_loaded_v2(&root, None, &narrow);
    let prepared = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        GraphBuildPlan::default(),
        &codebase_config,
        &narrow,
        &visible,
    )
    .unwrap();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    // The public prepared API may receive a different config after graph prep.
    let error = crate::codebase::rules::run_check_with_config_and_facts_and_playwright(
        crate::codebase::rules::PreparedRulesCheck {
            session: crate::codebase::analysis_session::AnalysisSession::disabled(),
            root: &root,
            config_path: Some(&config_path),
            tsconfig_path: None,
            shared: &shared,
            prepared_playwright: None,
            config: &requested,
            prepared_graph: Some(&prepared),
            prepared_tsconfig: &tsconfig,
            prepared_tsconfig_catalog: &catalog,
            prepared_vitest_projects: None,
            prepared_playwright_projects: None,
            inferred_roots: None,
            sources: None,
        },
    )
    .unwrap_err();
    assert!(
        format!("{error:#}").contains("missing graph facts"),
        "{error:#}"
    );
}
