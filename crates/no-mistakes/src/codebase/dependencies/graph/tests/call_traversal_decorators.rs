#[test]
fn call_traversal_reaches_static_decorators_without_guessing_computed_decorators() {
    let (root, graph) = call_fixture_graph();
    let decorators = root.join("src/decorator-reachability.mts");
    let targets = graph.call_traces(
        &[symbol(&decorators, "outer")],
        CallTraversal::Direct,
        None,
    );

    for name in ["classDecorator", "memberDecorator"] {
        assert!(
            targets.iter().any(|trace| {
                has_symbol(
                    &trace.target,
                    &root.join("src/decorator-target.mts"),
                    name,
                )
            }),
            "{name} must be reached through its static decorator; traces={targets:#?}; sites={:#?}",
            graph
                .resolved_call_sites()
                .iter()
                .filter(|site| site.file == decorators)
                .collect::<Vec<_>>()
        );
    }
    assert!(!targets.iter().any(|trace| {
        has_symbol(
            &trace.target,
            &root.join("src/dynamic-decorator-target.mts"),
            "dynamicDecorator",
        )
    }));
    let sites = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| site.file == decorators)
        .collect::<Vec<_>>();
    let factory = sites
        .iter()
        .find(|site| site.source_callee == "classDecorator")
        .expect("decorator factory call must remain named");
    assert!(sites.iter().any(|site| {
        site.source_callee == "<unknown>"
            && site.line == factory.line
            && site.caller_id == factory.caller_id
    }), "the returned decorator application is a distinct unknown call: {sites:#?}");
}
