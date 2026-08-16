use criterion::{black_box, Criterion, Throughput};
use no_mistakes::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes::codebase::dependencies::NodeId;
use no_mistakes::codebase::ts_resolver::load_tsconfig;
use no_mistakes::codebase::ts_source::discover_visible_paths;
use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::{Path, PathBuf};

/// Step-up corpus for visitor-fusion and outer-graph-parallelism gates.
/// Sized above the 14-file core-analysis fixture and below the 246-file
/// large-graph-monorepo acceptance fixture so CI benches stay in-process.
pub(super) const EXPECTED_SOURCE_FILES: usize = 75;
pub(super) const EXPECTED_GRAPH_FILES: usize = 82;
pub(super) const EXPECTED_FORWARD_DEPS: usize = 52;
pub(super) const EXPECTED_REVERSE_DEPENDENTS: usize = 28;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/graph-gates")
        .canonicalize()
        .expect("graph-gates performance fixture should exist")
}

fn source_files(root: &Path) -> Vec<PathBuf> {
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

fn file_nodes(root: &Path, rels: &[&str]) -> Vec<NodeId> {
    rels.iter()
        .map(|rel| NodeId::File(root.join(rel)))
        .collect()
}

fn build_graph(
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
            DepGraph::build_with_plan_and_config(
                root,
                config,
                GraphBuildPlan::all(),
                Some(config_path),
            )
            .expect("graph-gates build should succeed")
        })
}

fn canonical_graph_snapshot(graph: &DepGraph) -> Vec<String> {
    let mut nodes = graph.all_files().cloned().collect::<Vec<_>>();
    nodes.sort();
    let mut rows = nodes
        .iter()
        .map(|node| format!("{node:?}"))
        .collect::<Vec<_>>();
    for node in &nodes {
        let mut deps = graph
            .deps_of(std::slice::from_ref(node), None, None)
            .into_iter()
            .map(|entry| format!("{entry:?}"))
            .collect::<Vec<_>>();
        deps.sort();
        let mut dependents = graph
            .dependents_of(std::slice::from_ref(node), None, None)
            .into_iter()
            .map(|entry| format!("{entry:?}"))
            .collect::<Vec<_>>();
        dependents.sort();
        rows.push(format!("fwd {node:?} {deps:?}"));
        rows.push(format!("rev {node:?} {dependents:?}"));
    }
    rows
}

pub(super) fn bench_graph_gates(c: &mut Criterion) {
    let root = fixture_root();
    let files = source_files(&root);
    let config = load_tsconfig(&root.join("tsconfig.json")).expect("graph-gates tsconfig");
    let config_path = root.join(".no-mistakes.yml");
    let plan = TsFactPlan::imports_and_symbols();

    let facts_preflight = collect_ts_facts(&files, plan);
    assert_eq!(facts_preflight.len(), EXPECTED_SOURCE_FILES);
    assert!(facts_preflight
        .values()
        .all(|facts| facts.operational_error.is_none() && facts.parse_error.is_none()));

    let serial = build_graph(&root, &config, &config_path, 1);
    let parallel = build_graph(&root, &config, &config_path, 4);
    assert_eq!(
        canonical_graph_snapshot(&serial),
        canonical_graph_snapshot(&parallel),
        "serial and 4-thread graph builds must be byte-identical"
    );
    let preflight = parallel;
    let forward_roots = file_nodes(
        &root,
        &[
            "apps/web/src/entry.tsx",
            "apps/api/src/entry.ts",
            "apps/worker/src/entry.ts",
            "scripts/orchestrate.ts",
        ],
    );
    let reverse_roots = file_nodes(
        &root,
        &[
            "packages/core/src/core-0.ts",
            "packages/data/src/records/data-0.ts",
            "packages/ui/src/components/Card0.tsx",
            "packages/queue-factory/src/index.ts",
        ],
    );
    let deps = preflight.deps_of(&forward_roots, None, None);
    let dependents = preflight.dependents_of(&reverse_roots, None, None);
    assert_eq!(preflight.all_files().count(), EXPECTED_GRAPH_FILES);
    assert_eq!(deps.len(), EXPECTED_FORWARD_DEPS);
    assert_eq!(dependents.len(), EXPECTED_REVERSE_DEPENDENTS);

    let mut group = c.benchmark_group("graph_gates");
    group.throughput(Throughput::Elements(EXPECTED_SOURCE_FILES as u64));
    group.bench_function("facts_extract", |b| {
        b.iter(|| black_box(collect_ts_facts(black_box(&files), black_box(plan))));
    });
    group.bench_function("all_domains_build", |b| {
        b.iter(|| {
            black_box(
                DepGraph::build_with_plan_and_config(
                    black_box(&root),
                    black_box(&config),
                    black_box(GraphBuildPlan::all()),
                    Some(black_box(&config_path)),
                )
                .expect("graph-gates build should succeed"),
            )
        });
    });
    group.bench_function("forward_reverse_query", |b| {
        b.iter(|| {
            let deps = preflight.deps_of(black_box(&forward_roots), None, None);
            let dependents = preflight.dependents_of(black_box(&reverse_roots), None, None);
            black_box((deps.len(), dependents.len()))
        });
    });
    group.finish();
}
