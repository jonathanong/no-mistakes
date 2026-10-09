use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn per_item_graph_projection_reuses_each_source_and_parse() {
    use crate::codebase::dependencies::graph::{DepGraph, PreparedCheckFactGraphBuildRequest};
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/fixture");
    let materialized = crate::test_support::materialize_saved_fixture(&saved);
    let root = crate::codebase::ts_resolver::normalize_path(materialized.path());
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new_observed(
        &root,
        Some(Arc::clone(&observer)),
    );
    let visible = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &visible).unwrap();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, &root, &visible, &sources,
    )
    .unwrap();
    let catalog = super::run::prepared_tsconfig_catalog(
        &root,
        None,
        &tsconfig,
        &visible,
        &sources,
        Some(&config),
    );
    let graph_plan = super::try_canonical_graph_plan(&config).unwrap().unwrap();
    let codebase = crate::codebase::config::config_from_loaded_v2(&root, None, &config);
    let prepared = crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
        &root,
        graph_plan,
        &codebase,
        &config,
        &snapshot,
        crate::codebase::test_filter::TestFileFilter::new(&root, &config),
    )
    .unwrap();
    let (mut demand, context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            &root, graph_plan, &prepared,
        );
    demand.per_item_calls = true;
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible);
    crate::ast::begin_parse_count(&root);
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session, &root, (files.clone(), files.clone()),
        crate::codebase::check_facts::CheckFactPlan { graph: demand, graph_context: context, ..Default::default() },
        None, Arc::clone(&sources),
    );
    let graph = DepGraph::build_with_prepared_check_facts_and_session(
        PreparedCheckFactGraphBuildRequest {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: &catalog,
            plan: graph_plan,
            files,
            config_path: None,
            facts: &facts,
            prepared: &prepared,
        },
        session,
    )
    .unwrap();
    let before = sources.physical_read_count();
    let findings =
        super::query_reached_per_item::check_with_graph(&root, &config, &graph, &facts).unwrap();
    assert!(!findings.is_empty());
    assert_eq!(
        sources.physical_read_count(),
        before,
        "policy projection must borrow facts without reading sources"
    );
    let parses = crate::ast::finish_parse_count(&root);
    let reads = observer.source_read_snapshot();
    for name in ["src/entry.mts", "src/helpers.mts", "src/disabled.mts"] {
        let path = root.join(name);
        assert_eq!(parses.get(&path), Some(&1), "{parses:#?}");
        assert_eq!(reads.get(&path), Some(&1), "{reads:#?}");
    }
}
