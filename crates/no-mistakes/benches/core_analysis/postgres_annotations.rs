use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{check_json, check_json_observed};
use std::collections::BTreeSet;
use std::path::PathBuf;

const ENTRY_COUNT: usize = 12;
const SOURCE_COUNT: u64 = 15;
const ENTRY_SOURCE: &str =
    include_str!("../../../../fixtures/performance/postgres-annotations/src/entry-00.mts");
const DENSE_ENTRY_SOURCE: &str =
    include_str!("../../../../fixtures/performance/postgres-branch-copies/src/entry-00.mts");

pub(super) fn bench_postgres_annotations(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-annotations")
        .canonicalize()
        .expect("PostgreSQL annotation performance fixture should exist");
    let preflight = check_json(&root).expect("annotation benchmark check should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&preflight).expect("annotation report should be JSON");
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().expect("rule findings array");
    assert_eq!(findings.len(), ENTRY_COUNT, "one unsafe query per endpoint");
    let safe_line = ENTRY_SOURCE
        .lines()
        .position(|line| line.ends_with("// safe-query"))
        .expect("saved safe-query marker") as u64
        + 1;
    let unsafe_line = ENTRY_SOURCE
        .lines()
        .position(|line| line.ends_with("// unsafe-query"))
        .expect("saved unsafe-query marker") as u64
        + 1;
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding["line"].as_u64() == Some(safe_line))
            .count(),
        0,
        "all twelve annotated queries pass"
    );
    let expected_files = (0..ENTRY_COUNT)
        .map(|index| format!("src/entry-{index:02}.mts"))
        .collect::<BTreeSet<_>>();
    let actual_files = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], "postgres-require-query-annotation");
            assert_eq!(finding["line"].as_u64(), Some(unsafe_line));
            finding["file"].as_str().expect("finding file").to_string()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_files, expected_files);

    let (observed, diagnostics) =
        check_json_observed(&root, true).expect("observed annotation check should succeed");
    assert_eq!(observed, preflight);
    assert_eq!(
        diagnostics.work["parse.files"], SOURCE_COUNT,
        "shared helper fanout must parse each source once per invocation"
    );

    let dense_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-branch-copies")
        .canonicalize()
        .expect("saved dense branch fixture should exist");
    let dense_output = check_json(&dense_root).expect("dense branch check should succeed");
    let dense_report: serde_json::Value = serde_json::from_str(&dense_output).unwrap();
    assert_eq!(dense_report["warnings"], serde_json::json!([]));
    assert_eq!(dense_report["advisories"], serde_json::json!([]));
    let dense_safe_line = DENSE_ENTRY_SOURCE
        .lines()
        .position(|line| line.ends_with("// safe-query"))
        .expect("saved safe-query marker") as u64
        + 1;
    let dense_unsafe_line = DENSE_ENTRY_SOURCE
        .lines()
        .position(|line| line.ends_with("// unsafe-query"))
        .expect("saved unsafe-query marker") as u64
        + 1;
    let dense_findings = dense_report["rules"].as_array().unwrap();
    assert_eq!(dense_findings.len(), ENTRY_COUNT);
    assert!(dense_findings.iter().all(|finding| {
        finding["line"].as_u64() == Some(dense_unsafe_line)
            && finding["rule"] == "postgres-require-query-annotation"
    }));
    assert!(dense_findings
        .iter()
        .all(|finding| finding["line"].as_u64() != Some(dense_safe_line)));
    let dense_files = dense_findings
        .iter()
        .map(|finding| finding["file"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(dense_files, expected_files);

    let mut group = c.benchmark_group("postgres_annotations");
    group.throughput(Throughput::Elements((ENTRY_COUNT * 2) as u64));
    group.bench_function("branching_helpers_12_importers", |b| {
        // Each iteration owns a fresh request: immutable summaries and branch
        // state are prepared again instead of reusing warmed invocation caches.
        b.iter(|| {
            black_box(check_json(black_box(&root)).expect("annotation check should succeed"))
        });
    });
    group.bench_function("dense_branch_state_12_importers", |b| {
        b.iter(|| {
            black_box(check_json(black_box(&dense_root)).expect("dense check should succeed"))
        });
    });
    group.finish();
}
