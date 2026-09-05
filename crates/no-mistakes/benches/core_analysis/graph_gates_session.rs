use super::support::{traversal_snapshot, EXPECTED_GRAPH_NODES};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::dependencies::graph::DepGraph;
use no_mistakes::codebase::ts_resolver::TsConfig;
use std::path::Path;
use std::sync::Arc;

pub(super) fn bench_graph_gates_session(
    c: &mut Criterion,
    root: &Path,
    config: &TsConfig,
    config_path: &Path,
    session: Arc<AnalysisSession>,
    unprepared: &DepGraph,
) {
    let session_preflight = DepGraph::build_with_plan_and_config_and_session(
        root,
        config,
        super::support::gate_plan(),
        Some(config_path),
        Arc::clone(&session),
    )
    .expect("graph-gates session preflight should succeed");
    assert_eq!(
        traversal_snapshot(&session_preflight),
        traversal_snapshot(unprepared),
        "session-path graph build must preserve traversal order"
    );
    drop(session_preflight);

    let mut session_group = c.benchmark_group("graph_gates_build_session");
    session_group.throughput(Throughput::Elements(EXPECTED_GRAPH_NODES as u64));
    for threads in [1usize, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("graph-gates rayon pool");
        session_group.bench_with_input(BenchmarkId::from_parameter(threads), &threads, |b, _| {
            b.iter(|| {
                pool.install(|| {
                    black_box(
                        DepGraph::build_with_plan_and_config_and_session(
                            black_box(root),
                            black_box(config),
                            black_box(super::support::gate_plan()),
                            Some(black_box(config_path)),
                            black_box(Arc::clone(&session)),
                        )
                        .expect("graph-gates session build should succeed"),
                    )
                })
            });
        });
    }
    session_group.finish();
}
