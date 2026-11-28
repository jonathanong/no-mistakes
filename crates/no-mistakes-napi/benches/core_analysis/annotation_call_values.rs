use super::shard;
use criterion::{black_box, Criterion};
use no_mistakes::benchmark_support::check_json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const RULE: &str = "postgres-require-query-annotation";
const MISSING_ANNOTATION: &str = "query execution must start with a /* name */ annotation";
const ENTRY_SOURCE: &str =
    include_str!("../../../../fixtures/performance/annotation-fact-scale/src/entry-00.mts");
const REFRESH_SOURCE: &str = include_str!(
    "../../../../fixtures/performance/annotation-scalar-call-values/scalar-refresh/root.mts"
);
const OPAQUE_SOURCE: &str = include_str!(
    "../../../../fixtures/performance/annotation-scalar-call-values/scalar-opaque/root.mts"
);

pub(super) fn bench_annotation_call_values(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/performance");
    let workloads = [
        (
            "shared_callback_payload",
            "annotation-fact-scale",
            (0..64)
                .map(|index| format!("src/entry-{index:02}.mts"))
                .collect(),
            finding_line(ENTRY_SOURCE, "expected-finding"),
        ),
        (
            "scalar_refresh",
            "annotation-scalar-call-values/scalar-refresh",
            BTreeSet::from(["root.mts".to_string()]),
            finding_line(REFRESH_SOURCE, "SELECT id FROM widgets"),
        ),
        (
            "scalar_opaque",
            "annotation-scalar-call-values/scalar-opaque",
            BTreeSet::from(["root.mts".to_string()]),
            finding_line(OPAQUE_SOURCE, "SELECT id FROM widgets"),
        ),
    ];
    let mut group = c.benchmark_group("annotation_call_values");
    for (name, fixture, expected_files, expected_line) in workloads {
        let root = fixtures
            .join(fixture)
            .canonicalize()
            .expect("saved annotation fixture should exist");
        preflight(&root, &expected_files, expected_line);
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(check_json(black_box(&root)).expect("annotation check should succeed"))
            });
        });
    }
    group.finish();
}

fn finding_line(source: &str, marker: &str) -> u64 {
    source
        .lines()
        .position(|line| line.contains(marker))
        .expect("saved finding source marker") as u64
        + 1
}

fn preflight(root: &Path, expected_files: &BTreeSet<String>, expected_line: u64) {
    let output = check_json(root).expect("annotation preflight should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("annotation report JSON");
    assert_eq!(
        report["warnings"],
        serde_json::json!([]),
        "{}",
        root.display()
    );
    assert_eq!(
        report["advisories"],
        serde_json::json!([]),
        "{}",
        root.display()
    );
    let findings = report["rules"]
        .as_array()
        .expect("annotation findings array");
    assert_eq!(findings.len(), expected_files.len(), "{}", root.display());
    let files = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert_eq!(finding["line"].as_u64(), Some(expected_line));
            assert!(finding["message"]
                .as_str()
                .is_some_and(|message| message.ends_with(MISSING_ANNOTATION)));
            finding["file"].as_str().expect("finding file").to_string()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(&files, expected_files);
}
