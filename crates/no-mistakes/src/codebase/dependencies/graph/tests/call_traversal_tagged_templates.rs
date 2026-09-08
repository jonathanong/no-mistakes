#[test]
fn call_traversal_resolves_named_tags_without_guessing_dynamic_tags() {
    let (root, graph) = call_fixture_graph();
    let source = root.join("src/tagged-templates.mts");
    let traces = graph.call_traces(
        &[symbol(&source, "taggedTemplates")],
        CallTraversal::Direct,
        None,
    );

    assert!(traces
        .iter()
        .any(|trace| has_symbol(&trace.target, &source, "localTag")));
    assert!(traces.iter().any(|trace| {
        has_symbol(
            &trace.target,
            &root.join("src/tagged-template-target.mts"),
            "importedTag",
        )
    }));
    assert!(traces
        .iter()
        .any(|trace| has_symbol(&trace.target, &source, "dynamicTag")));
    assert_eq!(
        traces
            .iter()
            .filter(|trace| has_symbol(&trace.target, &source, "localTag"))
            .count(),
        1,
        "the dynamically returned tag must not create a second localTag edge"
    );
    assert_eq!(
        graph
            .resolved_call_sites()
            .iter()
            .filter(|site| site.file == source && site.source_callee == "<unknown>")
            .count(),
        2,
        "computed and dynamically produced tags remain unresolved instead of gaining edges"
    );
}
