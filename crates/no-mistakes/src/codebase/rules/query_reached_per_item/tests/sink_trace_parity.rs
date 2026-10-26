use super::*;
use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, GraphBuildPlan};

#[test]
fn targeted_traces_preserve_all_targets_display_roots_and_exclusions() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&root().with_file_name("sink-paths-fixture"));
    let graph = DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let file = root.join("src/entry.mts");
    for symbol in [
        "start",
        "startAlias",
        "selfSink",
        "transactionOnly",
        "absent",
    ] {
        let root = NodeId::symbol(&file, symbol);
        let all = graph.call_traces(std::slice::from_ref(&root), CallTraversal::Transitive, None);
        let targets = all
            .iter()
            .map(|trace| &trace.target)
            .collect::<crate::fx::FxHashSet<_>>();
        for excluded in [
            Vec::new(),
            graph.expand_call_roots(&[CallRoot::Function {
                file: file.clone(),
                symbol: "left".into(),
            }]),
        ] {
            let excluded_set = excluded.iter().collect();
            let mut actual = graph.call_traces_to_sinks_excluding(&root, &targets, &excluded_set);
            actual.sort_by(|a, b| {
                (&a.root, &a.target, &a.nodes).cmp(&(&b.root, &b.target, &b.nodes))
            });
            assert_eq!(
                actual,
                graph.call_traces_excluding(
                    std::slice::from_ref(&root),
                    CallTraversal::Transitive,
                    None,
                    &excluded
                )
            );
            assert!(graph
                .call_traces_to_sinks_excluding(&root, &Default::default(), &excluded_set)
                .is_empty());
        }
    }
}

#[test]
fn alias_and_multiple_candidate_roots_retain_exact_reaches() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&root().with_file_name("sink-paths-fixture"));
    let graph = DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let file = root.join("src/entry.mts");
    let expand = |symbol: &str| {
        graph.expand_call_roots(&[CallRoot::Function {
            file: file.clone(),
            symbol: symbol.into(),
        }])
    };
    assert_eq!(expand("startAlias"), expand("start"));
    let roots = ["startAlias", "selfSink", "transactionOnly"]
        .into_iter()
        .flat_map(expand)
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    let mut sinks = super::super::traces::SinkMap::new();
    for site in graph.resolved_call_sites() {
        if matches!(site.source_callee.as_str(), "read" | "tx.query") {
            sinks
                .entry(source_node(site))
                .or_default()
                .push((site, site.source_callee == "tx.query"));
        }
    }
    for excluded in [Vec::new(), expand("left"), expand("shared")] {
        let actual = super::super::traces::reachable(&graph, &roots, &sinks, &excluded);
        let expected = super::sink_paths::legacy_reachable(&graph, &roots, &sinks, &excluded);
        assert!(actual == expected);
        assert!(actual.values().all(Option::is_some));
    }
}

#[test]
fn physical_multi_target_call_preserves_reached_repository_sink() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &root().with_file_name("alternatives-fixture"),
    );
    let graph = DepGraph::build_with_plan(
        &root,
        &crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            calls: true,
            effect_selectors: true,
            ..Default::default()
        },
    )
    .unwrap();
    let site = graph
        .resolved_call_sites()
        .iter()
        .find(|site| {
            site.file == root.join("src/entry.mts") && site.source_callee == "receiver.transaction"
        })
        .unwrap();
    // This supported replacement retains both an external transaction target
    // and an aliased repository helper, rather than collapsing either to Unknown.
    assert_eq!(site.targets().len(), 2, "{site:#?}");
    assert!(!site.has_unknown_target());
    let roots = site.target_nodes().cloned().collect::<Vec<_>>();
    assert_eq!(roots.len(), 1);
    let mut sinks = super::super::traces::SinkMap::new();
    for site in graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.source_callee == "read")
    {
        sinks
            .entry(source_node(site))
            .or_default()
            .push((site, false));
    }
    let actual = super::super::traces::reachable(&graph, &roots, &sinks, &[]);
    assert!(actual == super::sink_paths::legacy_reachable(&graph, &roots, &sinks, &[]));
    assert!(actual
        .values()
        .next()
        .unwrap()
        .as_ref()
        .is_some_and(|reach| !reach.transaction));
}
