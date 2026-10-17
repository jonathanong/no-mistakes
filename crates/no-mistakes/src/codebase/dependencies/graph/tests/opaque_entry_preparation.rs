use super::*;

fn build(name: &str, original: bool) -> (serde_json::Value, ReplacementProbeSnapshot) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
    benchmark_probe_with_original_entries(original, || {
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
        (full, benchmark_probe_replacements())
    })
}

#[test]
fn opaque_entry_preparation_preserves_every_graph_field_and_raw_edge() {
    let mut avoided = 0;
    for name in [
        "value-flow-replacement-uncertain-execution",
        "value-flow-repeated-replacement-controls",
        "value-flow-foreign-parameter-timing",
        "value-flow-replacement-scale-16",
        "value-flow-replacement-scale-256",
        "value-flow-replacement-cold",
    ] {
        let (old, old_probe) = build(name, true);
        let (new, new_probe) = build(name, false);
        println!(
            "OPAQUE_ENTRY_ORACLE {}",
            serde_json::json!({"fixture":name,"old_full":old,"new_full":new,"old_probe":old_probe,"new_probe":new_probe})
        );
        assert_eq!(old, new, "all canonical public graph fields: {name}");
        assert_eq!(old_probe.incoming, new_probe.incoming, "incoming: {name}");
        let mut old_edges = old_probe.admission_raw_edges.clone();
        let mut new_edges = new_probe.admission_raw_edges.clone();
        old_edges.sort();
        new_edges.sort();
        assert_eq!(old_edges, new_edges, "raw edge multiplicity: {name}");
        assert_eq!(old_probe.rhs_attempts, new_probe.rhs_attempts);
        assert_eq!(old_probe.rhs_resolutions, new_probe.rhs_resolutions);
        assert_eq!(old_probe.rhs_known, new_probe.rhs_known);
        assert_eq!(old_probe.opaque_groups, new_probe.opaque_groups);
        assert_eq!(
            old_probe.entry_resolutions,
            new_probe.entry_resolutions + new_probe.opaque_entry_resolutions_avoided,
            "actual omitted named resolutions: {name}"
        );
        avoided += new_probe.opaque_entry_resolutions_avoided;
    }
    // Saved opaque execution must reach the optimization, not just a cold path.
    assert!(avoided > 0);
}

#[test]
fn opaque_entry_work_guard_requires_saved_opaque_entry_pruning() {
    let original = std::env::var_os("NO_MISTAKES_OPAQUE_ENTRY_ORIGINAL").is_some();
    let (full, probe) = build("value-flow-replacement-uncertain-execution", original);
    println!(
        "OPAQUE_ENTRY_WORK {}",
        serde_json::json!({"full":full,"probe":probe,"original":original})
    );
    assert!(
        probe.opaque_entry_resolutions_avoided > 0,
        "saved opaque entry resolutions must be omitted"
    );
}
