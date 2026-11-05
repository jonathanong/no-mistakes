use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::dependencies::graph::GraphBuildPlan;
use std::path::PathBuf;

#[test]
fn called_malformed_import_is_incomplete_but_unused_import_is_not() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation"),
    );
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join("query.yml"))).unwrap();
    let files = [
        "query-entry.ts",
        "query-unused.ts",
        "recovered.ts",
        "query-recovered.ts",
        "query-barrel.ts",
        "query-star.ts",
        "query-barrel-unused.ts",
        "barrel.ts",
        "star-barrel.ts",
        "broken.ts",
    ]
    .map(|file| root.join(file))
    .to_vec();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = collect_check_facts(
        &root,
        files.clone(),
        CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                function_calls: true,
                per_item_calls: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            effect_selectors: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!facts
        .get_ts_facts(&root.join("query-entry.ts"))
        .unwrap()
        .per_item_calls
        .is_empty());
    let sites = graph.call_sites_in_file(&root.join("query-entry.ts"));
    assert!(
        sites
            .iter()
            .any(|site| site.source_callee == "load"
                && graph.failed_call_sources(site).next().is_some()),
        "{sites:?}"
    );
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let output = super::super::check_with_graph_and_session(
        &root,
        &config,
        &graph,
        &facts,
        Some(&sources),
        Some(&session),
    )
    .unwrap();
    assert!(output.is_empty());
    assert_eq!(session.check_failures().len(), 1);
    assert!(session.check_failures()[0]
        .file
        .as_ref()
        .unwrap()
        .ends_with("broken.ts"));
    for file in ["query-barrel.ts", "query-star.ts", "query-recovered.ts"] {
        let mut selected = config.clone();
        selected.rules[0].include = vec![file.to_string()];
        let scoped = session.scoped_checks();
        super::super::check_with_graph_and_session(
            &root,
            &selected,
            &graph,
            &facts,
            Some(&sources),
            Some(&scoped),
        )
        .unwrap();
        assert_eq!(
            scoped.check_failures().len(),
            1,
            "{file}: {:?}",
            scoped.check_failures()
        );
        let failed = if file == "query-recovered.ts" {
            "recovered.ts"
        } else {
            "broken.ts"
        };
        assert!(scoped.check_failures()[0]
            .file
            .as_ref()
            .unwrap()
            .ends_with(failed));
        if file == "query-recovered.ts" {
            assert!(graph
                .call_sites_in_file(&root.join(file))
                .iter()
                .any(|site| site.source_callee == "load" && site.target_nodes().next().is_some()));
        }
    }
    let mut unused = config.clone();
    unused.rules[0].include = vec!["query-unused.ts".to_string()];
    let session = session.scoped_checks();
    super::super::check_with_graph_and_session(
        &root,
        &unused,
        &graph,
        &facts,
        Some(&sources),
        Some(&session),
    )
    .unwrap();
    assert!(
        session.check_failures().is_empty(),
        "{:?}",
        session.check_failures()
    );
    unused.rules[0].include = vec!["query-barrel-unused.ts".to_string()];
    let session = session.scoped_checks();
    super::super::check_with_graph_and_session(
        &root,
        &unused,
        &graph,
        &facts,
        Some(&sources),
        Some(&session),
    )
    .unwrap();
    assert!(session.check_failures().is_empty());
    for config_file in ["query-allowed.yml", "query-exempt.yml"] {
        let selected =
            crate::config::v2::load_v2_config(&root, Some(&root.join(config_file))).unwrap();
        let scoped = session.scoped_checks();
        let findings = super::super::check_with_graph_and_session(
            &root,
            &selected,
            &graph,
            &facts,
            Some(&sources),
            Some(&scoped),
        )
        .unwrap();
        assert!(findings.is_empty(), "{config_file}: {findings:?}");
        let expected_failures = usize::from(config_file == "query-allowed.yml");
        assert_eq!(
            scoped.check_failures().len(),
            expected_failures,
            "{config_file}: {:?}",
            scoped.check_failures()
        );
    }
}

#[test]
fn reachable_call_chain_reports_deep_failed_sources_and_honors_exclusions() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/query-reachable-chain"),
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let path = root.join("query-chain.ts");
    let sites = graph.call_sites_in_file(&path);
    let entry = sites
        .iter()
        .find(|site| site.source_callee == "helper")
        .unwrap();
    let leaf = sites
        .iter()
        .find(|site| site.source_callee == "leaf")
        .unwrap();
    let error = require_reachable(&graph, [entry], &[], None).unwrap_err();
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .unwrap();
    assert_eq!(
        failure.file.as_deref(),
        Some(root.join("broken.ts").as_path())
    );
    let mut excluded: Vec<_> = leaf.target_nodes().cloned().collect();
    assert!(!excluded.is_empty());
    excluded.sort();
    require_reachable(&graph, [entry], &excluded, None).unwrap();
    let roots: Vec<_> = entry.target_nodes().cloned().collect();
    let reached = reachable_sources(&graph, roots, &[]);
    assert!(leaf.target_nodes().all(|node| reached.contains(node)));
}
