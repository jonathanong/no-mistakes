use super::*;

#[test]
fn static_class_members_resolve_without_guessing_function_object_members() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let source = root.join("src/static-class-members.mts");
    let calls = graph.deps_of(
        &[NodeId::file(source.clone())],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "Service/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "ExpressionService/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    for symbol in ["InternalExpressionService", "InternalExpressionService/run"] {
        assert!(
            calls.iter().any(|entry| {
                matches!(
                    &entry.node,
                    NodeId::Symbol { file, symbol: candidate, callable_id: Some(_), .. }
                        if file.as_ref() == source.as_path() && candidate.as_ref() == symbol
                ) && entry.via.contains(&EdgeKind::Call)
            }),
            "outward named-class call must resolve to {symbol}"
        );
    }
    let named_static = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "NamedExpressionService.run")
        .expect("outward static call");
    let internal_static = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "InternalExpressionService.run")
        .expect("internal static call");
    assert_eq!(named_static.target, internal_static.target);
    for source_callee in [
        "NamedExpressionAlias",
        "NamedExpressionAlias.run",
        "namedExpressionRun",
    ] {
        assert!(graph.resolved_call_sites().iter().any(|site| {
            site.source_callee == source_callee
                && matches!(
                    &site.target,
                    crate::codebase::dependencies::graph::ResolvedCallTarget::RepositoryFunction { scope, .. }
                        if scope == "InternalExpressionService"
                            || scope == "InternalExpressionService/run"
                )
        }), "named class alias call must resolve: {source_callee}");
    }
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.source_callee == "ReassignedExpression.run"
            && site.target
                == crate::codebase::dependencies::graph::ResolvedCallTarget::Unknown
    }));
    for (source_callee, scope) in [
        ("NestedPublic", "namedExpressionNest/NestedInternal"),
        ("NestedPublic.run", "namedExpressionNest/NestedInternal/run"),
    ] {
        assert!(graph.resolved_call_sites().iter().any(|site| {
            site.caller.as_deref() == Some("namedExpressionNest")
                && site.source_callee == source_callee
                && matches!(
                    &site.target,
                    crate::codebase::dependencies::graph::ResolvedCallTarget::RepositoryFunction { scope: candidate, .. }
                        if candidate == scope
                )
        }), "nested outward class call must resolve to {scope}");
    }
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "ExpressionService/instance"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "api/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));

    for symbol in ["Internal", "Internal/run"] {
        let identities = calls
            .iter()
            .filter_map(|entry| match &entry.node {
                NodeId::Symbol {
                    file,
                    symbol: candidate,
                    callable_id: Some(id),
                    ..
                } if file.as_ref() == source.as_path() && candidate.as_ref() == symbol => Some(*id),
                _ => None,
            })
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(
            identities.len(),
            2,
            "sibling named class expressions must retain distinct {symbol} identities"
        );
    }
}
