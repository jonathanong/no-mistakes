use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan};
use no_mistakes::codebase::dependencies::NodeId;
use no_mistakes::codebase::ts_source::discover_visible_paths;
use no_mistakes::codebase::ts_source::facts::TsFactMap;
use std::path::{Path, PathBuf};

pub(super) const EXPECTED_SOURCE_FILES: usize = 75;
pub(super) const EXPECTED_IMPORTS: usize = 159;
pub(super) const EXPECTED_SYMBOL_IMPORTS: usize = 125;
pub(super) const EXPECTED_SYMBOL_EXPORTS: usize = 138;
pub(super) const EXPECTED_SYMBOL_REFS: usize = 222;
pub(super) const EXPECTED_GRAPH_NODES: usize = 199;
pub(super) const EXPECTED_SYMBOL_NODES: usize = 111;
pub(super) const EXPECTED_QUEUE_EDGES: usize = 6;
pub(super) const EXPECTED_QUEUE_WORKER_EDGES: usize = 12;
pub(super) const EXPECTED_HTTP_EDGES: usize = 18;
pub(super) const EXPECTED_MARKDOWN_EDGES: usize = 4;
pub(super) const EXPECTED_ROUTE_EDGES: usize = 12;
pub(super) const EXPECTED_TEST_EDGES: usize = 3;
pub(super) const EXPECTED_PACKAGE_EDGES: usize = 7;
pub(super) const EXPECTED_SELECTOR_EDGES: usize = 1;
pub(super) const EXPECTED_WORKSPACE_EDGES: usize = 10;
pub(super) const EXPECTED_FORWARD_DEPS: usize = 123;
pub(super) const EXPECTED_REVERSE_DEPENDENTS: usize = 35;
pub(super) const FORWARD_ROOTS: &[&str] = &[
    "apps/web/src/entry.tsx",
    "apps/api/src/entry.ts",
    "apps/worker/src/entry.ts",
    "scripts/orchestrate.ts",
];
pub(super) const REVERSE_ROOTS: &[&str] = &[
    "packages/core/src/core-0.ts",
    "packages/data/src/records/data-0.ts",
    "packages/ui/src/components/Card0.tsx",
    "packages/queue-factory/src/index.ts",
];

pub(super) fn gate_plan() -> GraphBuildPlan {
    GraphBuildPlan::all().with_symbols(true)
}

pub(super) fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/graph-gates")
        .canonicalize()
        .expect("graph-gates performance fixture should exist")
}

pub(super) fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = discover_visible_paths(root)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx" | "mts")
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), EXPECTED_SOURCE_FILES);
    files
}

pub(super) fn file_nodes(root: &Path, rels: &[&str]) -> Vec<NodeId> {
    rels.iter()
        .map(|rel| NodeId::file(root.join(rel)))
        .collect()
}

pub(super) fn build_graph(
    root: &Path,
    config: &no_mistakes::codebase::ts_resolver::TsConfig,
    config_path: &Path,
    threads: usize,
) -> DepGraph {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("graph-gates rayon pool")
        .install(|| {
            DepGraph::build_with_plan_and_config(root, config, gate_plan(), Some(config_path))
                .expect("graph-gates build should succeed")
        })
}

pub(super) fn traversal_snapshot(graph: &DepGraph) -> Vec<String> {
    let mut nodes = graph.all_files().cloned().collect::<Vec<_>>();
    nodes.sort();
    let mut rows = nodes
        .iter()
        .map(|node| format!("{node:?}"))
        .collect::<Vec<_>>();
    for node in &nodes {
        let deps: Vec<_> = graph
            .deps_of(std::slice::from_ref(node), None, None)
            .into_iter()
            .map(|entry| format!("{entry:?}"))
            .collect();
        let dependents: Vec<_> = graph
            .dependents_of(std::slice::from_ref(node), None, None)
            .into_iter()
            .map(|entry| format!("{entry:?}"))
            .collect();
        rows.push(format!("fwd {node:?} {deps:?}"));
        rows.push(format!("rev {node:?} {dependents:?}"));
    }
    rows
}

pub(super) fn count_kind(graph: &DepGraph, kind: EdgeKind) -> usize {
    graph
        .all_files()
        .filter_map(|node| graph.dependencies_of_node(node))
        .flatten()
        .filter(|(_, edge)| *edge == kind)
        .count()
}

pub(super) fn fact_totals(facts: &TsFactMap) -> (usize, usize, usize, usize) {
    facts.values().fold(
        (0, 0, 0, 0),
        |(imports, symbol_imports, exports, refs), file| {
            (
                imports + file.imports.len(),
                symbol_imports
                    + file
                        .symbols
                        .as_ref()
                        .map(|symbols| symbols.imports.len())
                        .unwrap_or(0),
                exports
                    + file
                        .symbols
                        .as_ref()
                        .map(|symbols| symbols.exports.len())
                        .unwrap_or(0),
                refs + file.symbol_references.len(),
            )
        },
    )
}

pub(super) fn expect_count(label: &str, actual: usize, expected: usize) {
    assert_eq!(actual, expected, "{label} actual={actual}");
}
