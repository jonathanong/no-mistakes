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
    assert_eq!(
        project_all_relationship_edges(&fixture),
        RelationshipProjectionSummary {
            projected_edges: 32,
        }
    );
    let constructed = relationship_index_from_fixture(relationship_construction_fixture(32));
    assert_eq!(
        project_relationship_edges(&constructed),
        RelationshipProjectionSummary {
            projected_edges: 32,
        }
    );
}

#[test]
fn graph_gates_full_domain_and_check_preflight_counts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/graph-gates")
        .canonicalize()
        .expect("graph-gates performance fixture should exist");
    let files = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx" | "mts")
            )
        })
        .collect::<Vec<_>>();
    let (plan, context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
            &root,
            crate::codebase::dependencies::graph::GraphBuildPlan::all().with_symbols(true),
            Some(&root.join(".no-mistakes.yml")),
        );
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context(&files, plan, &context);
    let route_refs = facts
        .values()
        .map(|file| file.route_refs.len())
        .sum::<usize>();
    let backend_routes = facts
        .values()
        .map(|file| file.backend_routes.len())
        .sum::<usize>();
    let queue_usage = facts
        .values()
        .filter(|file| file.queue_usage.is_some())
        .count();
    let http_calls = facts
        .values()
        .map(|file| file.http_calls.len())
        .sum::<usize>();
    let process_spawns = facts
        .values()
        .map(|file| file.process_spawns.len())
        .sum::<usize>();
    let react = facts
        .values()
        .map(|file| file.react_components.len())
        .sum::<usize>();
    let check = check_json(&root).expect("graph-gates check should succeed");
    let check_value: serde_json::Value =
        serde_json::from_str(&check).expect("graph-gates check report should be JSON");
    assert_eq!(facts.len(), 75);
    assert_eq!(route_refs, 12);
    assert_eq!(backend_routes, 9);
    assert_eq!(queue_usage, 75);
    assert_eq!(http_calls, 13);
    assert_eq!(process_spawns, 4);
    assert_eq!(react, 19);
    assert_eq!(check_value.as_object().map(|value| value.len()), Some(7));
}

#[test]
fn scoped_resolver_fixture_caches_one_selection_per_importer() {
    let fixture = scoped_resolver_selection_fixture();

    assert_eq!(
        resolve_repeated_scoped_imports(&fixture, 64),
        ScopedResolverSelectionSummary {
            resolved: 64,
            selection_builds: 1,
        }
    );
}
