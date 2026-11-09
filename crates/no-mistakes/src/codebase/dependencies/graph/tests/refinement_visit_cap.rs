use super::super::value_flow::site_storage;
use super::*;

fn build(original: bool, calls: bool) -> (serde_json::Value, ReplacementProbeSnapshot, usize) {
    site_storage::with_cloning(original, || {
        let root = crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-visit-cap"));
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
            benchmark_probe_replacements(),
            benchmark_probe_value_flow_evaluations(),
        )
    })
    .0
}

#[test]
fn sparse_admission_caps_actual_revisits_without_changing_graph_or_incoming() {
    let (old, old_probe, old_work) = build(true, true);
    let (new, new_probe, new_work) = build(false, true);
    let admitted = new_probe.receiver_initial_admissions_without_target_node
        + new_probe.receiver_initial_admissions_with_target_node;
    let total = new["sites"].as_array().unwrap().len();
    println!("VISIT_CAP total={total} admitted={admitted} old={old_work} new={new_work}");
    assert!(total >= 32 && admitted * 16 < total);
    assert_eq!(old, new, "all public graph fields");
    assert_eq!(
        old_probe.incoming, new_probe.incoming,
        "incoming owners and physical calls"
    );
    assert_eq!(
        old_probe.admission_raw_edges, new_probe.admission_raw_edges,
        "physical edge order and multiplicity"
    );
    let callback = new["sites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|site| site["callee"] == "api.run")
        .unwrap();
    assert_eq!(callback["unknown"], true);
    assert!(callback["known"].as_array().unwrap().is_empty());
    assert!(
        new_work > admitted,
        "the cycle must actually revisit a site"
    );
    assert!(
        new_work <= admitted * (admitted + 2),
        "enabled visits plus disabled fallback"
    );
    assert_eq!(new_work, 5, "four cycling visits and one timer visit");
    assert!(
        old_work > new_work,
        "unrelated known sites cannot widen the cap"
    );
    assert_eq!(
        build(true, false).0,
        build(false, false).0,
        "imports-only control"
    );
}
