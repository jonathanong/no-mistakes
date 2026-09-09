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

#[test]
fn immutable_object_aliases_resolve_callable_members() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/immutable-object-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callThroughFacade")],
        CallTraversal::Direct,
        None,
    );

    assert_eq!(targets.len(), 1);
    assert!(has_symbol(&targets[0].target, &file, "target"));
}

#[test]
fn increment_removes_the_callable_alias_edge() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/updated-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callAfterUpdate")],
        CallTraversal::Direct,
        None,
    );

    assert!(targets.is_empty());
}

#[test]
fn duplicate_object_key_removes_the_overwritten_callable_edge() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/duplicate-object-key-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callDuplicateKey")],
        CallTraversal::Direct,
        None,
    );

    assert!(targets.is_empty());
}

#[test]
fn duplicate_object_key_keeps_the_last_callable_write() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/last-write-object-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callLastWrite")],
        CallTraversal::Direct,
        None,
    );

    assert_eq!(targets.len(), 1);
    assert!(has_symbol(&targets[0].target, &file, "later"));
}

#[test]
fn later_object_spread_removes_the_callable_member_edge() {
    let (root, graph) = call_fixture_graph();
    let file = root.join("src/later-spread-callable-alias.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "callAfterSpread")],
        CallTraversal::Direct,
        None,
    );

    assert!(targets.is_empty());
}
