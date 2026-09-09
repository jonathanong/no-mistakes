use super::*;

#[test]
fn destructured_callable_aliases_resolve_to_their_source_functions() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/destructured-callable-aliases.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callThroughDestructuredAliases")],
        CallTraversal::Direct,
        None,
    );

    assert_eq!(targets.len(), 2);
    assert!(
        targets
            .iter()
            .any(|target| has_symbol(&target.target, &file, "objectTarget"))
    );
    assert!(
        targets
            .iter()
            .any(|target| has_symbol(&target.target, &file, "arrayTarget"))
    );
}

#[test]
fn nested_object_member_reassignment_removes_the_callable_edge() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/reassigned-object-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callAfterReplacement")],
        CallTraversal::Direct,
        None,
    );

    assert!(targets.is_empty());
}
