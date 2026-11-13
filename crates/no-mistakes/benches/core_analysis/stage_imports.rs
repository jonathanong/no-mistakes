use super::shard;
use criterion::{black_box, BatchSize, Criterion, Throughput};
use no_mistakes::benchmark_support::check_json;
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::dependencies::graph::{
    CallRoot, CallTraversal, DepGraph, GraphBuildPlan,
};
use no_mistakes::codebase::ts_resolver::load_tsconfig;
use std::path::PathBuf;

pub(super) fn bench_stage_imports(c: &mut Criterion) {
    if !shard::should_run(shard::QUERY) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/stage-import-index-workload")
        .canonicalize()
        .expect("stage import workload should exist");
    let config = load_tsconfig(&root.join("tsconfig.json")).expect("stage import tsconfig");
    let config_path = root.join(".no-mistakes.yml");
    let plan = GraphBuildPlan {
        calls: true,
        effect_selectors: true,
        value_import_selectors: true,
        ..GraphBuildPlan::default()
    };
    let build = |session| {
        DepGraph::build_with_plan_and_config_and_session(
            &root,
            &config,
            plan,
            Some(&config_path),
            session,
        )
        .expect("stage import graph construction should succeed")
    };
    let preflight = build(AnalysisSession::disabled());
    let caller = root.join("src/caller.mts");
    assert_eq!(
        preflight
            .resolved_call_sites()
            .iter()
            .filter(|site| site.file == caller)
            .count(),
        48
    );
    let roots = [CallRoot::Function {
        file: caller,
        symbol: "cleanup".into(),
    }];
    let expanded = preflight.expand_call_roots(&roots);
    assert_eq!(expanded.len(), 1);
    assert_eq!(
        preflight
            .call_traces(&expanded, CallTraversal::Transitive, None)
            .len(),
        48
    );
    let report: serde_json::Value = serde_json::from_str(
        &check_json(&root).expect("stage import check preflight should succeed"),
    )
    .unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 48);
    assert!(findings
        .iter()
        .all(|finding| finding["rule"] == "query-reached-per-item"));

    let mut group = c.benchmark_group("stage_imports");
    group.throughput(Throughput::Elements(3_121));
    group.bench_function("fresh_session_discovery_parse_graph", |b| {
        b.iter_batched(
            AnalysisSession::disabled,
            |session| black_box(build(black_box(session))),
            BatchSize::SmallInput,
        );
    });
    group.bench_function("prepared_call_root_lookup", |b| {
        b.iter(|| black_box(preflight.expand_call_roots(black_box(&roots))));
    });
    group.bench_function("prepared_call_trace_projection", |b| {
        b.iter(|| {
            black_box(preflight.call_traces(black_box(&expanded), CallTraversal::Transitive, None))
        });
    });
    group.finish();
}
