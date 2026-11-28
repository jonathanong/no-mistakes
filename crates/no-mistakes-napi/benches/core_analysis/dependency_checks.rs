use super::shard;
use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{check_json, check_json_observed};
use std::path::PathBuf;

pub(super) fn bench_dependency_checks(c: &mut Criterion) {
    if !shard::should_run(shard::CHECK) {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/dependency-checks")
        .canonicalize()
        .expect("dependency check performance fixture should exist");
    let preflight = check_json(&root).expect("dependency check preflight should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&preflight).expect("dependency report should be JSON");
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().expect("rule findings array");
    assert_eq!(
        findings.len(),
        1,
        "one deliberately dev-only runtime import"
    );
    assert_eq!(findings[0]["rule"], "production-dependency-declarations");
    assert_eq!(findings[0]["file"], "packages/p12/local.ts");
    assert_eq!(findings[0]["line"], 1);
    assert_eq!(findings[0]["target"], "fixture-runtime");
    let (observed, _) =
        check_json_observed(&root, true).expect("observed dependency check should succeed");
    assert_eq!(observed, preflight);

    let mut group = c.benchmark_group("dependency_checks");
    group.throughput(Throughput::Elements(48));
    group.bench_function("24_packages_three_lockfile_applications", |b| {
        // Construct fresh request inputs so preparation and traversal costs
        // stay visible rather than warming caches between iterations.
        b.iter(|| {
            black_box(check_json(black_box(&root)).expect("dependency check should succeed"))
        });
    });
    group.finish();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/workspace-closure-paths")
        .canonicalize()
        .expect("workspace closure performance fixture should exist");
    let preflight = check_json(&root).expect("workspace closure preflight should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&preflight).expect("workspace closure report should be JSON");
    assert_eq!(report["warnings"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
    let findings = report["rules"].as_array().expect("rule findings array");
    assert_eq!(findings.len(), 64);
    let mut starts = std::collections::BTreeSet::new();
    let mut chains = std::collections::BTreeSet::new();
    for finding in findings {
        assert_eq!(finding["rule"], "forbidden-workspace-closure");
        assert_eq!(finding["target"], "fixture-denied");
        let chain = finding["import"]
            .as_str()
            .expect("workspace closure finding should contain a dependency chain");
        let start = chain
            .split_once(" -> ")
            .map(|(start, _)| start)
            .expect("workspace closure chain should include a dependency");
        starts.insert(start);
        chains.insert(chain);
    }
    let expected_starts: std::collections::BTreeSet<_> = std::iter::once("@perf/app".to_string())
        .chain(
            (0..31)
                .flat_map(|layer| ['a', 'b'].map(move |side| format!("@perf/p{layer:02}{side}"))),
        )
        .chain(std::iter::once("@perf/p31a".to_string()))
        .collect();
    assert_eq!(
        starts.len(),
        64,
        "each matching workspace start should be unique"
    );
    assert_eq!(
        chains.len(),
        64,
        "each workspace should produce a unique chain"
    );
    assert_eq!(
        starts,
        expected_starts.iter().map(String::as_str).collect(),
        "all workspace packages with a path to the forbidden dependency should match"
    );
    let expected_app_chain = std::iter::once("@perf/app".to_string())
        .chain((0..32).map(|layer| format!("@perf/p{layer:02}a")))
        .chain(std::iter::once("fixture-denied".to_string()))
        .collect::<Vec<_>>()
        .join(" -> ");
    assert!(
        findings
            .iter()
            .any(|finding| { finding["import"].as_str() == Some(expected_app_chain.as_str()) }),
        "the app should retain the first path through the 'a' branch"
    );
    let (observed, _) =
        check_json_observed(&root, true).expect("observed workspace closure check should succeed");
    assert_eq!(observed, preflight);

    let mut group = c.benchmark_group("workspace_closure_paths");
    group.throughput(Throughput::Elements(65));
    group.bench_function("65_workspace_starts_32_layer_diamonds", |b| {
        // One shared policy covers all workspaces, measuring independent BFS
        // starts over the same 32-layer diamond rather than duplicate checks.
        b.iter(|| black_box(check_json(black_box(&root)).expect("workspace check should succeed")));
    });
    group.finish();
}
