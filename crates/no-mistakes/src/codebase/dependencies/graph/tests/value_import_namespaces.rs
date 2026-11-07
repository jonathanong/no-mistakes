use super::*;

#[test]
fn namespace_import_selectors_retain_bare_aliases_and_exact_exports() {
    let (root, files, facts) = prepared();
    let graph = build(&root, files, &facts, true);
    let consumer = root.join("namespace-consumer.ts");
    let source = root.join("source.ts");
    for selector in [
        "grouped",
        "imported",
        "relayed",
        "throughStar",
        "direct",
        "nested.grouped",
    ] {
        assert_eq!(
            graph.imported_namespace(&consumer, selector),
            Some(source.clone()),
            "{selector}"
        );
        assert!(graph.namespace_exports_value(&source, &source, "statement"));
        assert!(graph.namespace_exports_value(&source, &source, "TABLES"));
        assert!(graph.imported_value(&consumer, selector).is_none());
    }
    assert_eq!(
        graph.imported_namespace(&consumer, "nested"),
        Some(root.join("namespace-star.ts"))
    );
    assert!(graph.namespace_exports_value(&root.join("namespace-star.ts"), &source, "statement"));
    assert_eq!(
        graph.imported_value(&consumer, "nested.grouped.statement"),
        Some((source.clone(), "statement".into()))
    );
    assert_eq!(
        graph.imported_value(&consumer, "explicitValue"),
        Some((root.join("other.ts"), "TABLES".into()))
    );
    for selector in ["unknown", "collided", "explicitValue", "erased"] {
        assert!(
            graph.imported_namespace(&consumer, selector).is_none(),
            "{selector}"
        );
    }
    let unrelated = graph.imported_namespace(&consumer, "unrelated").unwrap();
    assert!(!graph.namespace_exports_value(&unrelated, &source, "statement"));
    assert!(!graph.namespace_exports_value(&source, &source, "mutable"));
    assert!(!graph.namespace_exports_value(&source, &root.join("other.ts"), "TABLES"));
    let quoted = graph.imported_namespace(&consumer, "quoted").unwrap();
    assert!(graph.namespace_exports_value(&quoted, &source, "statement"));
}

#[test]
fn value_selector_demand_preserves_external_member_call_classification_in_mixed_plans() {
    let (root, files, facts) = prepared();
    let consumer = root.join("external-member-consumer.ts");
    for effect_selectors in [false, true] {
        let build = |value_import_selectors| {
            DepGraph::build_with_plan_file_list_and_check_facts(
                &root,
                &load_tsconfig(&root.join("tsconfig.json")).unwrap(),
                GraphBuildPlan {
                    calls: true,
                    imports: true,
                    effect_selectors,
                    value_import_selectors,
                    ..Default::default()
                },
                files.clone(),
                &facts,
            )
            .unwrap()
        };
        let ordinary = build(false);
        let selected = build(true);
        for file in &files {
            assert_eq!(
                ordinary.call_sites_in_file(file),
                selected.call_sites_in_file(file),
                "{file:?}, effects={effect_selectors}"
            );
        }
        assert!(ordinary
            .call_sites_in_file(&consumer)
            .iter()
            .any(|site| site.source_callee == "query" && site.has_unknown_target()));
        assert_eq!(
            selected.imported_value(&consumer, "settings"),
            Some((root.join("external-member.ts"), "settings".into()))
        );
    }
}

#[test]
fn namespace_query_rejects_unknown_and_colliding_edges() {
    let root = normalize_path(&fixture("value-import-selectors"));
    let consumer = root.join("namespace-consumer.ts");
    let namespace = NodeId::file(root.join("source.ts"));
    for extra in [
        (NodeId::file(root.join("other.ts")), EdgeKind::SymbolImport),
        (
            NodeId::symbol(root.join("source.ts"), "statement"),
            EdgeKind::SymbolImport,
        ),
        (NodeId::module("unknown"), EdgeKind::SymbolImportUnknown),
    ] {
        let graph = test_support::from_typed_maps(
            root.clone(),
            [(
                NodeId::symbol(&consumer, "grouped"),
                vec![(namespace.clone(), EdgeKind::SymbolImport), extra],
            )]
            .into_iter()
            .collect(),
            Default::default(),
        );
        assert!(graph.imported_namespace(&consumer, "grouped").is_none());
    }
}
