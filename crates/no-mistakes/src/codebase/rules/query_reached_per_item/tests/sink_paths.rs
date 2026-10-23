use super::super::traces::{reachable, SinkMap};
use super::*;
use crate::codebase::dependencies::graph::call_path_metrics_tests::take_call_path_count;
use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, GraphBuildPlan};

thread_local! {
    static LEGACY_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static USE_LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Frozen all-target consumer, used only as a behavioral oracle.
pub(crate) fn legacy_reachable(
    graph: &DepGraph,
    roots: &[NodeId],
    sinks: &SinkMap<'_>,
    excluded: &[NodeId],
) -> BTreeMap<NodeId, Option<Reach>> {
    LEGACY_CALLS.with(|calls| calls.set(calls.get() + 1));
    roots
        .iter()
        .map(|node| {
            let paths = std::iter::once((node.clone(), vec![node.clone()])).chain(
                graph
                    .call_traces_excluding(
                        std::slice::from_ref(node),
                        CallTraversal::Transitive,
                        None,
                        excluded,
                    )
                    .into_iter()
                    .map(|trace| (trace.target, trace.nodes)),
            );
            let values = paths
                .flat_map(|(target, path)| {
                    sinks
                        .get(&target)
                        .into_iter()
                        .flatten()
                        .map(move |(site, transaction)| {
                            let mut labels = path.iter().map(node_label).collect::<Vec<_>>();
                            labels.push(site.source_callee.clone());
                            Reach {
                                path: labels,
                                transaction: *transaction,
                            }
                        })
                })
                .collect::<Vec<_>>();
            let only_transaction =
                !values.is_empty() && values.iter().all(|value| value.transaction);
            let best = shortest(values.into_iter()).map(|mut value| {
                value.transaction = only_transaction;
                value
            });
            (node.clone(), best)
        })
        .collect()
}

#[test]
fn sink_paths_preserve_complete_findings_against_all_target_consumer() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        for fixture in [
            "sink-paths-fixture",
            "fixture",
            "targets-fixture",
            "callbacks-fixture",
            "alternatives-fixture",
        ] {
            let root = root().with_file_name(fixture);
            let actual = crate::codebase::rules::run_check(&root, None, None).unwrap();
            LEGACY_CALLS.with(|calls| calls.set(0));
            USE_LEGACY.with(|mode| mode.set(true));
            let expected = crate::codebase::rules::run_check(&root, None, None);
            USE_LEGACY.with(|mode| mode.set(false));
            assert!(
                LEGACY_CALLS.with(|calls| calls.get()) > 0,
                "oracle must run"
            );
            assert!(!actual.is_empty());
            assert_eq!(actual, expected.unwrap(), "{fixture}");
        }
    });
}

#[test]
fn production_reachable_reconstructs_only_reached_sink_paths() {
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
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: file.clone(),
        symbol: "start".into(),
    }]);
    assert_eq!(roots.len(), 1);
    let mut sinks = SinkMap::new();
    for site in graph.resolved_call_sites() {
        if matches!(site.source_callee.as_str(), "read" | "tx.query") {
            sinks
                .entry(source_node(site))
                .or_default()
                .push((site, site.source_callee == "tx.query"));
        }
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        for excluded_name in [None, Some("left"), Some("shared"), Some("start")] {
            let excluded = excluded_name
                .map(|symbol| {
                    graph.expand_call_roots(&[CallRoot::Function {
                        file: file.clone(),
                        symbol: symbol.into(),
                    }])
                })
                .unwrap_or_default();
            let all =
                graph.call_traces_excluding(&roots, CallTraversal::Transitive, None, &excluded);
            let expected_paths = all
                .iter()
                .filter(|trace| sinks.contains_key(&trace.target))
                .count();
            take_call_path_count();
            let actual = reachable(&graph, &roots, &sinks, &excluded);
            let constructed = take_call_path_count();
            // Counting inside production traversal catches falling back to the
            // all-target implementation even when every finding is identical.
            assert_eq!(constructed, expected_paths);
            assert!(actual == legacy_reachable(&graph, &roots, &sinks, &excluded));
            if excluded_name.is_none() {
                assert_eq!(constructed, 2);
                assert!(all.len() > constructed + 10);
                let reach = actual.values().next().unwrap().as_ref().unwrap();
                assert_eq!(reach.path, ["start", "left", "shared", "read"]);
                assert!(!reach.transaction);
            }
        }
    });
}
