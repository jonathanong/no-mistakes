use super::super::value_flow::site_storage;
use super::*;

fn build(
    name: &str,
    original: bool,
    calls: bool,
) -> (
    serde_json::Value,
    Option<ReplacementProbeSnapshot>,
    (usize, usize),
) {
    site_storage::with_original(original, || {
        let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
        let config = TsConfig {
            dir: root.clone(),
            paths: vec![],
            paths_dir: root.clone(),
            base_url: None,
        };
        let graph = DepGraph::build_with_plan(
            &root,
            &config,
            GraphBuildPlan {
                imports: true,
                calls,
                ..Default::default()
            },
        )
        .unwrap();
        let mut full = inherited_graph_snapshot::render(&graph, &root);
        inherited_graph_snapshot::normalize_fixture_root(&mut full, &root);
        (
            full,
            calls.then(benchmark_probe_replacements),
            site_storage::last(),
        )
    })
}

#[test]
fn scalar_refinement_removes_actual_nonempty_single_site_vectors() {
    let original = std::env::var_os("NO_MISTAKES_SITE_VECTOR_ORIGINAL").is_some();
    let (full, _, (vectors, scalars)) = build("value-flow-replacement-scale-16", original, true);
    println!(
        "SITE_STORAGE_WORK {}",
        serde_json::json!({"full":full,"vectors":vectors,"scalars":scalars,"original":original})
    );
    // The unchanged physical sixteen receiver calls must actually be refined.
    assert_eq!(full["sites"].as_array().unwrap().len(), 16);
    assert_eq!(vectors, 0, "one-element refinement Vec budget");
    assert!(
        scalars >= 16,
        "a disabled refinement path cannot satisfy the budget"
    );
}

#[test]
fn scalar_refinement_preserves_full_graph_incoming_and_physical_edges() {
    for name in [
        "value-flow-replacement-cold",
        "inline-member-precision-no-writes",
        "value-flow-replacement-scale-16",
        "value-flow-replacement-scale-256",
        "value-flow-foreign-parameter-timing",
        "value-flow-static-call-admission",
        "value-flow-repeated-replacement-controls",
        "value-flow-replacement-uncertain-execution",
    ] {
        for calls in [false, true] {
            let (old, old_probe, (old_vectors, old_scalars)) = build(name, true, calls);
            let (new, new_probe, (new_vectors, new_scalars)) = build(name, false, calls);
            assert_eq!(old, new, "all public graph fields: {name}/{calls}");
            assert_eq!(old_scalars, 0);
            assert_eq!(new_vectors, 0);
            assert_eq!(
                old_vectors, new_scalars,
                "same actual refinement cardinality: {name}/{calls}"
            );
            if let (Some(old), Some(new)) = (old_probe, new_probe) {
                assert_eq!(
                    old.incoming, new.incoming,
                    "all incoming owners and physical arguments: {name}"
                );
                // Keep order and multiplicity: sorting or deduplication could hide a lost edge.
                assert_eq!(
                    old.admission_raw_edges, new.admission_raw_edges,
                    "physical edge sequence: {name}"
                );
            } else {
                assert_eq!(
                    (old_vectors, new_scalars),
                    (0, 0),
                    "no-calls demand stays cold"
                );
            }
        }
    }
}

#[test]
fn scalar_refinement_mode_and_counts_stay_request_owned_under_rayon() {
    let (old, new) = std::thread::scope(|scope| {
        let old = scope.spawn(|| build("value-flow-replacement-scale-16", true, true));
        let new = scope.spawn(|| build("value-flow-replacement-scale-16", false, true));
        (old.join().unwrap(), new.join().unwrap())
    });
    assert_eq!(old.0, new.0);
    assert!(old.2 .0 >= 16);
    assert_eq!(old.2 .1, 0);
    assert_eq!(new.2 .0, 0);
    assert_eq!(old.2 .0, new.2 .1);
}

#[test]
fn in_place_refinement_removes_per_visit_whole_site_clones() {
    let original = std::env::var_os("NO_MISTAKES_SITE_CLONE_ORIGINAL").is_some();
    for (name, count) in [
        ("value-flow-replacement-scale-16", 16),
        ("value-flow-replacement-scale-256", 256),
    ] {
        let ((full, _, (_, visits)), clones) =
            site_storage::with_cloning(original, || build(name, false, true));
        assert_eq!(full["sites"].as_array().unwrap().len(), count);
        assert!(visits >= count, "actual saved hot reads must be refined");
        println!("SITE_OWNERSHIP_WORK {name} visits={visits} whole_site_clones={clones}");
        assert_eq!(clones, 0, "per-visit whole-site clone budget");
    }
}

#[test]
fn in_place_refinement_preserves_scalar_vector_incoming_and_physical_edges() {
    for name in [
        "value-flow-replacement-cold",
        "inline-member-precision-no-writes",
        "value-flow-replacement-scale-16",
        "value-flow-replacement-scale-256",
        "value-flow-foreign-parameter-timing",
        "value-flow-static-call-admission",
        "value-flow-repeated-replacement-controls",
        "value-flow-replacement-uncertain-execution",
    ] {
        for calls in [false, true] {
            let ((old, old_probe, old_work), old_clones) =
                site_storage::with_cloning(true, || build(name, false, calls));
            let ((new, new_probe, new_work), new_clones) =
                site_storage::with_cloning(false, || build(name, false, calls));
            let ((vector, vector_probe, _), _) =
                site_storage::with_cloning(true, || build(name, true, calls));
            assert_eq!(old, new, "all scalar graph fields {name}/{calls}");
            assert_eq!(vector, new, "all original vector fields {name}/{calls}");
            assert_eq!(old_work, new_work, "same refinement visits {name}/{calls}");
            assert_eq!(new_clones, 0);
            if let (Some(old), Some(new), Some(vector)) = (old_probe, new_probe, vector_probe) {
                assert_eq!(
                    old.incoming, new.incoming,
                    "incoming owners/physical arguments {name}"
                );
                assert_eq!(vector.incoming, new.incoming);
                assert_eq!(
                    old.admission_raw_edges, new.admission_raw_edges,
                    "raw edge order/multiplicity {name}"
                );
                assert_eq!(vector.admission_raw_edges, new.admission_raw_edges);
                assert_eq!(
                    old_clones, old_work.1,
                    "actual old whole-site clone per visit"
                );
            } else {
                assert_eq!((old_clones, new_work.1), (0, 0), "no-calls zero control");
            }
        }
    }
}
