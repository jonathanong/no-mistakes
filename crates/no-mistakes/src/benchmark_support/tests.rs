use super::*;
use serde_json::json;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/core-analysis")
        .canonicalize()
        .expect("performance fixture should exist")
}

#[test]
fn benchmark_adapters_preserve_output_with_and_without_observers() {
    let root = fixture_root();
    let plain_check = check_json(&root).expect("plain check should succeed");
    let (observed_check, check_diagnostics) =
        check_json_observed(&root, true).expect("observed check should succeed");
    assert_eq!(observed_check, plain_check);
    assert!(!check_diagnostics.work.is_empty());

    let options = json!({
        "root": root,
        "tsconfig": fixture_root().join("tsconfig.json"),
        "reports": [
            {
                "id": "dependencies",
                "type": "dependencies",
                "files": ["src/app.tsx"],
                "relationships": ["all"]
            }
        ]
    })
    .to_string();
    let plain_project =
        analyze_project_json(options.clone()).expect("plain project analysis should succeed");
    let (observed_project, project_diagnostics) =
        analyze_project_json_observed(options).expect("observed project analysis should succeed");
    assert_eq!(observed_project, plain_project);
    assert!(!project_diagnostics.work.is_empty());
}

#[test]
fn high_fanout_finalization_dedupes_and_preserves_canonical_order() {
    let first = high_fanout_finalization_fixture(32, 7);
    let second = high_fanout_finalization_fixture(32, 7);
    assert_eq!(
        finalize_high_fanout_adjacency(first.clone()),
        HighFanoutFinalizationSummary {
            canonical_edges: 32 * 7,
            forward_nodes: 32,
            reverse_nodes: 32,
        }
    );
    assert_eq!(
        high_fanout_finalization_signature(first),
        high_fanout_finalization_signature(second),
        "source-ordered finalization must not depend on HashMap iteration order"
    );
}

#[test]
fn high_fanout_finalization_emits_split_verbose_timings() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let fixture = high_fanout_finalization_fixture(32, 7);
    let _ = finalize_high_fanout_adjacency(fixture);
    drop(guard);

    let labels = observer
        .snapshot()
        .timings
        .into_iter()
        .map(|timing| timing.label)
        .collect::<Vec<_>>();
    assert!(labels.contains(&"graph.canonical_flatten".to_string()));
    assert!(labels.contains(&"graph.ordinal_construction".to_string()));
}

#[test]
fn production_graph_fixture_exercises_finalization_and_selector_append() {
    let fixture = production_graph_fixture(32, 7);
    assert_eq!(
        finalize_production_graph(fixture.clone()),
        ProductionGraphSummary {
            canonical_edges: 32 * 7,
            selector_appended_edges: 0,
        }
    );
    assert_eq!(
        append_production_selectors(fixture),
        ProductionGraphSummary {
            canonical_edges: 32 * 7,
            selector_appended_edges: 32 * 7,
        }
    );
}

#[test]
fn relationship_projection_fixture_deduplicates_typed_public_collisions() {
    let fixture = relationship_projection_fixture(32);
    assert_eq!(
        project_relationship_edges(&fixture),
        RelationshipProjectionSummary {
            projected_edges: 32,
        }
    );
}
