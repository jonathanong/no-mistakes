use super::fixtures::{
    fixture_root, impacted_args, EXPECTED_IMPACTED_CHECKS, EXPECTED_MULTI_REPORT_RESOLVER_KEYS,
};
use super::shard;
use criterion::{black_box, Criterion};
use no_mistakes::benchmark_support;
use no_mistakes::impacted_checks::generate_impacted_checks;
use serde_json::json;
use std::path::PathBuf;

const EXPECTED_GRAPH_GATES_CHECK_KEYS: usize = 7;

pub(super) fn bench_finite_set_membership(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let scope: Vec<PathBuf> = (0..50_000)
        .map(|index| PathBuf::from(format!("src/generated/{index}.ts")))
        .collect();
    let candidates: Vec<PathBuf> = (0..1_000)
        .map(|index| scope[(index * 37) % scope.len()].clone())
        .collect();
    c.bench_function("check/finite_set_membership_50k_scope_1k_candidates", |b| {
        b.iter(|| {
            criterion::black_box(
                no_mistakes::codebase::check_facts::ordered_path_intersection(
                    criterion::black_box(&candidates),
                    criterion::black_box(&scope),
                ),
            )
        });
    });
}

pub(super) fn bench_aggregate_and_multi_report(c: &mut Criterion) {
    if shard::should_run(shard::CHECK) {
        let root = fixture_root();
        let check_preflight =
            benchmark_support::check_json(&root).expect("check preflight should succeed");
        let check_value: serde_json::Value =
            serde_json::from_str(&check_preflight).expect("check report should be JSON");
        assert_eq!(check_value.as_object().map(|value| value.len()), Some(7));

        c.bench_function("aggregate/all_configured_check_domains", |b| {
            b.iter(|| {
                black_box(
                    benchmark_support::check_json(black_box(&root)).expect("check should succeed"),
                )
            });
        });

        let gates_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/performance/graph-gates")
            .canonicalize()
            .expect("graph-gates performance fixture should exist");
        let gates_check = benchmark_support::check_json(&gates_root)
            .expect("graph-gates check preflight should succeed");
        let gates_value: serde_json::Value =
            serde_json::from_str(&gates_check).expect("graph-gates check report should be JSON");
        assert_eq!(
            gates_value.as_object().map(|value| value.len()),
            Some(EXPECTED_GRAPH_GATES_CHECK_KEYS),
            "graph-gates check top-level keys drifted"
        );
        c.bench_function("aggregate/graph_gates_check", |b| {
            b.iter(|| {
                black_box(
                    benchmark_support::check_json(black_box(&gates_root))
                        .expect("graph-gates check should succeed"),
                )
            });
        });
    }

    if !shard::should_run(shard::QUERY) {
        return;
    }
    let options = json!({
        "root": fixture_root(),
        "tsconfig": fixture_root().join("tsconfig.json"),
        "reports": [
            {"id": "dependencies", "type": "dependencies", "files": ["src/app.tsx"], "relationships": ["all"]},
            {"id": "dependents", "type": "dependents", "files": ["packages/core/src/index.ts"]},
            {"id": "symbol-dependents", "type": "dependents", "files": [{"file": "packages/core/src/index.ts", "symbol": "CoreValue"}], "relationships": ["all"]},
            {"id": "symbols", "type": "symbols", "files": ["src/app.tsx"], "include": "both"}
        ]
    })
    .to_string();
    let multi_preflight = benchmark_support::analyze_project_json(options.clone())
        .expect("multi-report preflight should succeed");
    let multi_value: serde_json::Value =
        serde_json::from_str(&multi_preflight).expect("multi-report output should be JSON");
    assert_eq!(multi_value["reports"].as_array().map(Vec::len), Some(4));
    let (observed_multi, multi_diagnostics) =
        benchmark_support::analyze_project_json_observed(options.clone())
            .expect("observed multi-report preflight should succeed");
    assert_eq!(observed_multi, multi_preflight);
    assert_eq!(multi_diagnostics.work["graph.builds"], 1);
    assert!(multi_diagnostics.work["graph.reuses"] >= 1);
    assert_eq!(multi_diagnostics.work["symbol_index.builds"], 1,);
    assert_eq!(
        multi_diagnostics.work["resolver.computations"],
        EXPECTED_MULTI_REPORT_RESOLVER_KEYS,
    );
    assert_eq!(
        multi_diagnostics.work["resolver.unique_keys"],
        EXPECTED_MULTI_REPORT_RESOLVER_KEYS,
    );
    assert!(
        multi_diagnostics.work["resolver.computations"]
            < multi_diagnostics.work["resolver.requests"]
    );

    c.bench_function("aggregate/reused_multi_report", |b| {
        b.iter(|| {
            black_box(
                benchmark_support::analyze_project_json(black_box(options.clone()))
                    .expect("multi-report should succeed"),
            )
        });
    });
}

pub(super) fn bench_impacted_checks(c: &mut Criterion) {
    if !shard::should_run(shard::TESTS_PLAN) {
        return;
    }
    let root = fixture_root();
    let preflight = generate_impacted_checks(&impacted_args(&root))
        .expect("impacted-checks preflight should succeed");
    assert_eq!(preflight.checks.len(), EXPECTED_IMPACTED_CHECKS);

    c.bench_function("impacted_checks/configured", |b| {
        b.iter(|| {
            black_box(
                generate_impacted_checks(black_box(&impacted_args(&root)))
                    .expect("impacted checks should succeed"),
            )
        });
    });
}
