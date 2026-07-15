use super::fixtures::{
    fixture_root, EXPECTED_CHECK_MANIFEST_PARSES, EXPECTED_CHECK_RESOLVER_KEYS,
    EXPECTED_CHECK_SOURCE_READS,
};
use criterion::{black_box, BenchmarkId, Criterion};
use no_mistakes::benchmark_support;
use no_mistakes::diagnostics::DiagnosticsSnapshot;
use std::path::Path;

fn observed_check(root: &Path, mode: &str) -> (String, Option<DiagnosticsSnapshot>) {
    match mode {
        "disabled" => (
            benchmark_support::check_json(root).expect("disabled check should succeed"),
            None,
        ),
        "timings" => {
            let (output, snapshot) = benchmark_support::check_json_observed(root, false)
                .expect("timed check should succeed");
            (output, Some(snapshot))
        }
        "verbose" => {
            let (output, snapshot) = benchmark_support::check_json_observed(root, true)
                .expect("verbose check should succeed");
            (output, Some(snapshot))
        }
        _ => unreachable!("benchmark mode is fixed"),
    }
}

pub(super) fn bench_observer_overhead(c: &mut Criterion) {
    let root = fixture_root();
    let (disabled_output, disabled_snapshot) = observed_check(&root, "disabled");
    let (timed_output, timed_snapshot) = observed_check(&root, "timings");
    let (verbose_output, verbose_snapshot) = observed_check(&root, "verbose");
    assert_eq!(disabled_output, timed_output);
    assert_eq!(disabled_output, verbose_output);
    assert!(disabled_snapshot.is_none());
    let timed_snapshot = timed_snapshot.expect("timed observer should have a snapshot");
    assert!(!timed_snapshot.timings.is_empty());
    assert!(timed_snapshot.work.is_empty());
    let verbose_snapshot = verbose_snapshot.expect("verbose observer should have a snapshot");
    assert!(!verbose_snapshot.timings.is_empty());
    assert_eq!(
        verbose_snapshot.work["source.reads"],
        EXPECTED_CHECK_SOURCE_READS
    );
    assert_eq!(
        verbose_snapshot.work["manifest.parses"],
        EXPECTED_CHECK_MANIFEST_PARSES
    );
    assert_eq!(verbose_snapshot.work["manifest.requests"], 8);
    assert_eq!(verbose_snapshot.work["manifest.cache_hits"], 4);
    assert_eq!(verbose_snapshot.work["parse.requests"], 14);
    assert_eq!(verbose_snapshot.work["parse.files"], 13);
    // All configured check domains share one canonical union graph.
    assert_eq!(verbose_snapshot.work["graph.builds"], 1);
    assert_eq!(
        verbose_snapshot.work["resolver.computations"],
        EXPECTED_CHECK_RESOLVER_KEYS,
    );
    assert_eq!(
        verbose_snapshot.work["resolver.unique_keys"],
        EXPECTED_CHECK_RESOLVER_KEYS
    );

    let mut group = c.benchmark_group("observer_overhead");
    for mode in ["disabled", "timings", "verbose"] {
        group.bench_with_input(BenchmarkId::from_parameter(mode), &mode, |b, mode| {
            b.iter(|| black_box(observed_check(black_box(&root), black_box(mode))));
        });
    }
    group.finish();
}
