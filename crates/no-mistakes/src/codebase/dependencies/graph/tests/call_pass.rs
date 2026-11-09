use super::*;

fn assembly_graph() -> DepGraph {
    let root = fixture("call-site-assembly");
    DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths: vec![],
            paths_dir: root.clone(),
            base_url: None,
        },
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap()
}

#[test]
fn call_pass_assembly_reserves_one_exact_site_slab_without_losing_occurrences() {
    let graph = assembly_graph();
    let expected = graph.resolved_call_sites();
    assert!(
        !expected.len().is_power_of_two(),
        "fixture must expose geometric spare capacity"
    );
    assert!(expected.len() > 8);
    assert!(expected
        .iter()
        .any(|site| site.invocation == InvocationKind::Construct));
    assert!(expected
        .iter()
        .any(|site| site.target == ResolvedCallTarget::Unknown));
    let mut parts = vec![CallPass::default()];
    parts.extend(expected.iter().map(|site| CallPass {
        sites: vec![site.clone()],
        ..CallPass::default()
    }));
    parts.push(CallPass::default());
    let merged = merge_call_passes(parts);
    assert_eq!(merged.sites, expected);
    assert_eq!(
        merged.sites.capacity(),
        expected.len(),
        "assembly must not geometrically grow the whole-project site slab"
    );
}

#[test]
fn call_pass_assembly_handles_empty_and_single_batches() {
    let empty = merge_call_passes(Vec::new());
    assert!(empty.sites.is_empty());
    assert_eq!(empty.sites.capacity(), 0);
    let graph = assembly_graph();
    let expected = graph.resolved_call_sites();
    let merged = merge_call_passes(vec![CallPass {
        sites: expected.to_vec(),
        ..CallPass::default()
    }]);
    assert_eq!(merged.sites, expected);
    assert_eq!(merged.sites.capacity(), expected.len());
}
