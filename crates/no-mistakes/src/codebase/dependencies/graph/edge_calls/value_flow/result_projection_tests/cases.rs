use super::*;

fn physical_site() -> ResolvedCallSite {
    run("value-flow-replacement-scale-16", false)
        .0
        .0
        .into_iter()
        .next()
        .unwrap()
}
fn old_named_target(name: &str) -> ResolvedCallTarget {
    ResolvedCallTarget::Global { name: name.into() }
}
#[test]
fn projected_results_preserve_change_even_with_identical_or_missing_nodes() {
    let mut site = physical_site();
    site.target = old_named_target("new");
    for node in [
        None,
        Some(NodeId::callable(&site.file, "same", CallableId(7))),
    ] {
        site.target_node = node.clone();
        let previous = PreviousCallNodes::project((old_named_target("old"), node.clone()), &site);
        assert!(
            previous.changed(),
            "different names must trigger the old update even for equal nodes"
        );
        assert_eq!(
            previous.target_nodes().collect::<Vec<_>>(),
            node.iter().collect::<Vec<_>>()
        );
        let unchanged = PreviousCallNodes::project((site.target.clone(), node), &site);
        assert!(!unchanged.changed());
        assert_eq!(unchanged.target_nodes().count(), 0);
    }
    let unidentified = NodeId::symbol(&site.file, "unidentified");
    assert_eq!(
        node_function_keys(InvocationKind::Call, std::iter::once(&unidentified)).count(),
        0
    );
    let previous = PreviousCallNodes::project(
        (
            site.target.clone(),
            Some(NodeId::symbol(&site.file, "other")),
        ),
        &site,
    );
    assert!(previous.changed(), "node-only changes count too");
}

#[test]
fn alternative_projection_keeps_deduplicated_old_nodes_and_membership_boundary() {
    let mut site = physical_site();
    site.target = ResolvedCallTarget::Unknown;
    site.target_node = None;
    let node = NodeId::callable(&site.file, "callee", CallableId(5));
    let candidate = |name: &str| ResolvedCallCandidate {
        target: KnownCallTarget::Global { name: name.into() },
        node: Some(node.clone()),
    };
    // Distinct target spellings must not duplicate a physical incoming argument.
    let targets = std::sync::Arc::new(ResolvedCallAlternatives::new(
        vec![candidate("a"), candidate("b")],
        true,
    ));
    let other = NodeId::callable(&site.file, "second", CallableId(6));
    let mut candidates = targets.known().to_vec();
    candidates.push(ResolvedCallCandidate {
        target: KnownCallTarget::Global { name: "c".into() },
        node: Some(other.clone()),
    });
    let targets = std::sync::Arc::new(ResolvedCallAlternatives::new(candidates, true));
    let mut expected = [node, other];
    expected.sort_unstable();
    let pointer = std::sync::Arc::as_ptr(&targets);
    let projected =
        PreviousCallNodes::project((ResolvedCallTarget::Alternatives(targets), None), &site);
    let PreviousCallNodes::Alternatives(old) = &projected else {
        panic!("retain existing alternatives ownership");
    };
    assert_eq!(
        std::sync::Arc::as_ptr(old),
        pointer,
        "move the Arc without allocating a replacement set"
    );
    assert_eq!(
        projected.target_nodes().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        node_function_keys(InvocationKind::Call, projected.target_nodes()).collect::<Vec<_>>(),
        expected
            .iter()
            .map(|node| match node {
                NodeId::Symbol {
                    callable_id: Some(id),
                    ..
                } => (site.file.clone(), *id),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        node_function_keys(InvocationKind::Membership, projected.target_nodes()).count(),
        0
    );
}
