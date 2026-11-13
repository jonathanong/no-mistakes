#[test]
fn absent_other_external_and_cyclic_star_exports_never_become_namespace_constructors() {
    let (root, graph) =
        super::class_bases::build_fixture("namespace-import-boundaries", true, true);
    let class = graph
        .class_declarations()
        .iter()
        .find(|class| class.scope == "Item" && class.namespace.as_deref() == Some("Known"))
        .unwrap();
    let sites = graph.call_sites_in_file(&root.join("src/consumer.ts"));
    assert_eq!(sites.len(), 7, "{sites:#?}");
    let good = sites
        .iter()
        .find(|site| site.source_callee == "Good.Item")
        .unwrap();
    assert_eq!(good.target_node.as_ref(), Some(&class.node()), "{good:#?}");
    assert!(!good.has_unknown_target(), "{good:#?}");
    for callee in [
        "Absent.Item",
        "Other.Item",
        "External.Item",
        "Cycle.Item",
        "Missing.Item",
        "Module.Known",
    ] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert_ne!(
            site.target_node.as_ref(),
            Some(&class.node()),
            "{callee}: {site:#?}"
        );
    }
    let callers = super::namespace_members::callers(&root, &graph, class);
    assert_eq!(
        callers,
        vec![("src/consumer.ts".to_owned(), Some("good".to_owned()))]
    );
}
