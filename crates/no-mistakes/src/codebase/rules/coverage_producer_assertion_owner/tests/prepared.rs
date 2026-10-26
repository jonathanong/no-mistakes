use super::super::*;
use std::sync::Arc;

#[test]
fn prepared_coverage_graph_reuses_sources_and_parse_and_keeps_call_output_stable() {
    use crate::codebase::dependencies::graph::{
        DepGraph, PreparedCheckFactGraphBuildRequest, TsFactLookup,
    };
    let saved = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner");
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
    let catalog = crate::codebase::rules::run::prepared_tsconfig_catalog(
        &root,
        None,
        &tsconfig,
        &visible,
        &sources,
        Some(&config),
    );
    let graph_plan = crate::codebase::rules::try_canonical_graph_plan(&config)
        .unwrap()
        .unwrap();
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
    let (demand, context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            &root, graph_plan, &prepared,
        );
    assert!(demand.call_eligibility);
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
        session.clone(),
    )
    .unwrap();
    // A library caller may supply a prepared graph together with invalid rule
    // options: diagnostics must preserve configuration errors at this boundary.
    for name in [
        "empty-producers.yml",
        "typo.yml",
        "invalid-policy.yml",
        "invalid-glob.yml",
        "invalid-contract-producer.yml",
        "invalid-contract-owner.yml",
    ] {
        let invalid =
            serde_yaml::from_str::<NoMistakesConfig>(&sources.read_path(&root.join(name)).unwrap())
                .unwrap();
        assert!(
            crate::codebase::rules::coverage_producer_assertion_owner::check(
                &root, &invalid, &graph, &facts, &sources
            )
            .is_err(),
            "{name}"
        );
    }
    // Structured call facts may omit the opaque source-local identity. Keep
    // their stable public owner instead of promoting the call to module scope.
    let mut legacy = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.caller.is_some())
        .unwrap()
        .clone();
    legacy.caller_id = None;
    assert!(
        matches!(source(&legacy), NodeId::Symbol { symbol, callable_id: None, .. } if symbol.as_ref() == legacy.caller.as_deref().unwrap())
    );
    let before = sources.physical_read_count();
    let diagnostics = crate::codebase::rules::coverage_producer_assertion_owner::check(
        &root, &config, &graph, &facts, &sources,
    )
    .unwrap();
    assert!(!diagnostics.findings.is_empty());
    assert_eq!(
        sources.physical_read_count(),
        before,
        "policy projection must borrow facts without reading sources"
    );
    let producer = config::Function {
        file: "src/producers.mts".into(),
        symbol: "crossProcess".into(),
    }
    .resolve(&root, &graph, &facts)
    .unwrap();
    assert!(graph
        .dependencies_of_node(&producer)
        .unwrap()
        .iter()
        .any(|(_, kind)| *kind == EdgeKind::AssertionOwner));
    let dead = config::Function {
        file: "src/producers.mts".into(),
        symbol: "deadProcess".into(),
    }
    .resolve(&root, &graph, &facts)
    .unwrap();
    assert!(graph
        .dependencies_of_node(&dead)
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::AssertionOwner));
    let call_only = DepGraph::build_with_prepared_check_facts_and_session(
        PreparedCheckFactGraphBuildRequest {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: &catalog,
            plan: crate::codebase::dependencies::graph::GraphBuildPlan {
                calls: true,
                ..Default::default()
            },
            files: facts.graph_file_universe().to_vec(),
            config_path: None,
            facts: &facts,
            prepared: &prepared,
        },
        session.clone(),
    )
    .unwrap();
    assert_eq!(graph.resolved_call_sites(), call_only.resolved_call_sites());
    assert_eq!(
        graph.call_traces(
            std::slice::from_ref(&producer),
            crate::codebase::dependencies::graph::CallTraversal::Transitive,
            None
        ),
        call_only.call_traces(
            &[producer],
            crate::codebase::dependencies::graph::CallTraversal::Transitive,
            None
        )
    );
    super::source_failures::assert_policies(&root, &config, &facts, &sources, |lookup| {
        DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
            crate::codebase::dependencies::graph::PreparedGraphBuild {
                root: &root,
                tsconfig: &tsconfig,
                tsconfig_catalog: Some(&catalog),
                plan: graph_plan,
                graph_files: &crate::codebase::dependencies::graph::GraphFiles::from_files(
                    facts.graph_file_universe().to_vec(),
                ),
                config_path: None,
                facts: Some(lookup),
                prepared: &prepared,
                import_resolution_cache: None,
                dotnet_facts: None,
                swift_facts: None,
                visible_paths: Some(&snapshot),
            },
            session.clone(),
        )
        .unwrap()
    });
    let parses = crate::ast::finish_parse_count(&root);
    let reads = observer.source_read_snapshot();
    for path in facts.graph_file_universe() {
        if facts.get_ts_facts(path).is_none() {
            continue;
        }
        assert_eq!(parses.get(path), Some(&1), "{parses:#?}");
        assert_eq!(reads.get(path), Some(&1), "{reads:#?}");
    }
}
