use super::*;

#[test]
fn resource_reachability_keeps_exported_member_scopes_and_resolves_top_level_dots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("extractor-final-semantics.ts");
    for symbols in [false, true] {
        let facts = collect_ts_facts(
            std::slice::from_ref(&consumer),
            TsFactPlan {
                function_calls: true,
                resources: true,
                symbols,
                ..TsFactPlan::default()
            },
        );
        let file_facts = facts
            .get(&consumer)
            .expect("fixture source must produce TS facts");
        assert_eq!(file_facts.symbols.is_some(), symbols);
        let reachable = reachable_function_scopes(file_facts);
        assert!(file_facts
            .function_calls
            .iter()
            .any(|call| call.caller.is_none() && call.callee == "api.load"));
        assert!(reachable.contains("api/load"));
        for scope in ["api/load", "Service/load"] {
            let call = file_facts
                .resource_calls
                .iter()
                .find(|call| call.function_scope.as_deref() == Some(scope))
                .expect("fixture contains a resource call in {scope}");
            assert!(resource_is_reachable(call, file_facts, &reachable));
        }
    }
}

#[test]
fn symbol_mode_does_not_widen_exported_aggregate_resource_reachability() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    let consumer = root.join("exported-member-consumer.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&consumer),
        TsFactPlan {
            function_calls: true,
            resources: true,
            symbols: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts
        .get(&consumer)
        .expect("fixture source must produce TS facts");
    assert!(file_facts.symbols.is_some());
    let reachable = reachable_function_scopes(file_facts);
    let direct = file_facts
        .resource_calls
        .iter()
        .find(|call| call.function_scope.as_deref() == Some("api/load"))
        .expect("fixture contains a direct exported member resource");
    assert!(resource_is_reachable(direct, file_facts, &reachable));
    let nested = file_facts
        .resource_calls
        .iter()
        .find(|call| call.function_scope.as_deref() == Some("api/load/unused"))
        .expect("fixture contains an uncalled nested helper resource");
    assert!(!resource_is_reachable(nested, file_facts, &reachable));

    let nested_member = file_facts
        .resource_calls
        .iter()
        .find(|call| call.function_scope.as_deref() == Some("api/nested/load"))
        .expect("fixture contains a nested exported aggregate member resource");
    assert!(resource_is_reachable(nested_member, file_facts, &reachable));

    let nested_target = root.join("resources/exported-object-nested.txt");
    let unused_target = root.join("resources/exported-object-unused.txt");
    let (edges, _, diagnostics) = collect_resource_edges(
        &root,
        std::slice::from_ref(&consumer),
        &facts,
        &[nested_target.clone(), unused_target.clone()],
    );
    assert!(diagnostics.is_empty());
    assert!(edges.contains(&(
        NodeId::File(consumer.clone()),
        NodeId::File(nested_target),
        EdgeKind::Resource,
    )));
    assert!(!edges
        .iter()
        .any(|edge| matches!(&edge.1, NodeId::File(path) if path == &unused_target)));
}

#[test]
fn generic_default_resource_roots_do_not_depend_on_symbol_collection() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("resource-impact"));
    for (source, scope) in [
        ("exported-default-direct.ts", "default"),
        ("exported-default-wrapped.ts", "default"),
        ("exported-default-named-class.ts", "Service/load"),
        ("exported-default-anonymous-class.ts", "default/load"),
    ] {
        let consumer = root.join(source);
        for symbols in [false, true] {
            let facts = collect_ts_facts(
                std::slice::from_ref(&consumer),
                TsFactPlan {
                    function_calls: true,
                    resources: true,
                    symbols,
                    ..TsFactPlan::default()
                },
            );
            let file_facts = facts
                .get(&consumer)
                .expect("fixture source must produce TS facts");
            assert_eq!(file_facts.symbols.is_some(), symbols);
            let reachable = reachable_function_scopes(file_facts);
            let call = file_facts
                .resource_calls
                .iter()
                .find(|call| call.function_scope.as_deref() == Some(scope))
                .expect("default export must use the matching resource scope");
            assert!(resource_is_reachable(call, file_facts, &reachable));
        }
    }
}
