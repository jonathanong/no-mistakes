use super::*;

fn build(name: &str, original: bool) -> (serde_json::Value, ReplacementProbeSnapshot, usize) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
    let root = if name == "value-flow-foreign-parameter-timing" {
        std::env::var_os("NO_MISTAKES_INHERITED_COUNTER_FIXTURE")
            .map(PathBuf::from)
            .unwrap_or(root)
    } else {
        root
    };
    benchmark_probe_with_original_admission(original, || {
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
                calls: true,
                ..Default::default()
            },
        )
        .unwrap();
        let mut full = inherited_graph_snapshot::render(&graph, &root);
        inherited_graph_snapshot::normalize_fixture_root(&mut full, &root);
        let probe = benchmark_probe_replacements();
        assert_eq!(
            probe.receiver_original_admission, original,
            "mode must be captured in the request cache before Rayon work"
        );
        (full, probe, benchmark_probe_value_flow_evaluations())
    })
}
fn edge_multiset(probe: &ReplacementProbeSnapshot) -> Vec<String> {
    let mut edges = probe.admission_raw_edges.clone();
    edges.sort(); // No deduplication: the physical edge multiplicity is the invariant.
    edges
}

#[test]
fn admission_work_guard_rejects_unnecessary_known_node_refinement() {
    let original = std::env::var_os("NO_MISTAKES_ADMISSION_TEST_ORIGINAL").is_some();
    let (full, probe, evaluations) = build("value-flow-foreign-parameter-timing", original);
    println!(
        "ADMISSION_WORK_GUARD {}",
        serde_json::json!({"full": full, "probe": probe, "evaluations": evaluations})
    );
    // The same assertion must actually fail with the original admission toggle.
    assert!(probe.receiver_skipped_direct_callable_admissions > 0);
}

#[test]
fn admission_oracle_preserves_full_graphs_incoming_ownership_and_edge_multiplicity() {
    for name in [
        "value-flow-foreign-parameter-timing",
        "value-flow-static-call-admission",
        "value-flow-replacement-scale-16",
        "value-flow-replacement-scale-256",
    ] {
        let (old, old_probe, old_work) = build(name, true);
        let (new, new_probe, new_work) = build(name, false);
        println!(
            "ADMISSION_ORACLE {}",
            serde_json::json!({"fixture": name, "old_full": old, "new_full": new, "old_probe": old_probe, "new_probe": new_probe, "old_evaluations": old_work, "new_evaluations": new_work})
        );
        assert_eq!(old, new, "all public graph fields: {name}");
        assert_eq!(
            old_probe.incoming, new_probe.incoming,
            "all private incoming inputs/owners/physical sources: {name}"
        );
        assert_eq!(
            edge_multiset(&old_probe),
            edge_multiset(&new_probe),
            "raw edge multiplicity: {name}"
        );
        assert_eq!(
            old_probe.receiver_initial_admissions_without_target_node,
            new_probe.receiver_initial_admissions_without_target_node,
            "baseline admissions: {name}"
        );
        assert_eq!(
            old_probe.receiver_initial_admissions_with_target_node,
            new_probe.receiver_initial_admissions_with_target_node
                + new_probe.receiver_skipped_direct_callable_admissions,
            "only exact direct callables skipped: {name}"
        );
        assert_eq!(old_probe.receiver_skipped_direct_callable_admissions, 0);
        if name == "value-flow-foreign-parameter-timing"
            || name == "value-flow-static-call-admission"
        {
            assert!(
                new_probe.receiver_skipped_direct_callable_admissions > 0,
                "saved positive work control: {name}"
            );
            assert!(new_work < old_work, "actual refinement work: {name}");
        }
        if name == "value-flow-foreign-parameter-timing" {
            for callee in [
                "nestedRepeated",
                "repeatedSaved",
                "forwarded",
                "lexicalEarly.get",
                "foreignOnly.get",
            ] {
                let site = new["sites"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|site| site["callee"] == callee)
                    .unwrap();
                assert_eq!(
                    site["unknown"], true,
                    "2022 correction must remain: {callee}"
                );
                assert_eq!(site["known"].as_array().unwrap().len(), 0);
            }
            assert!(
                new_probe.incoming.iter().any(|row| row.physical.len() > 1),
                "physical caller multiplicity remains observable"
            );
        }
    }
}
