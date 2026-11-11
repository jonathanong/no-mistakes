use super::*;

#[test]
fn scenario_ownership_projection_preserves_ordinary_call_queries() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-scenario-table"),
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let build = |assertion_owners| {
        DepGraph::build_with_plan(
            &root,
            &config,
            GraphBuildPlan {
                calls: true,
                imports: true,
                workspace: true,
                assertion_owners,
                ..Default::default()
            },
        )
        .unwrap()
    };
    let ordinary = build(false);
    let ownership = build(true);
    assert_eq!(
        ordinary.resolved_call_sites(),
        ownership.resolved_call_sites()
    );
    let main = ordinary
        .resolved_call_sites()
        .iter()
        .find(|site| {
            site.file == root.join("scripts/runner/run.mts")
                && site.caller.as_deref() == Some("main")
                && site.source_callee == "scenario.run"
        })
        .unwrap();
    assert!(main.has_unknown_target());
    let projected = ownership
        .assertion_call_sites()
        .iter()
        .find(|site| {
            site.file == main.file && site.offset == main.offset && site.caller_id == main.caller_id
        })
        .unwrap();
    assert!(!projected.has_unknown_target());
    assert!(projected
        .target_nodes()
        .any(|node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "runX")));
    for caller in ["mixedFirst", "mixedLast"] {
        let mixed = ownership
            .assertion_call_sites()
            .iter()
            .find(|site| site.caller.as_deref() == Some(caller) && site.source_callee == "row.run")
            .unwrap();
        assert!(mixed.has_unknown_target());
        assert!(mixed.target_nodes().any(
            |node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "runX")
        ));
    }
    let source = NodeId::callable(&main.file, "main", main.caller_id.unwrap());
    let calls = |graph: &DepGraph| {
        graph
            .dependencies_of_node(&source)
            .into_iter()
            .flatten()
            .filter(|(_, kind)| *kind == EdgeKind::Call)
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(calls(&ordinary), calls(&ownership));
    assert!(ownership
        .dependencies_of_node(&source)
        .unwrap()
        .iter()
        .any(|(_, kind)| *kind == EdgeKind::AssertionOwner));
}
