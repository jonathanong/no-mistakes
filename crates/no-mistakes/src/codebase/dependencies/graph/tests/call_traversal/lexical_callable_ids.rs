use super::*;

#[test]
fn duplicate_local_display_scopes_keep_their_lexical_callable_ids() {
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
    let file = root.join("src/duplicate-local-call-targets.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: file.clone(),
        symbol: "run".to_string(),
    }]);

    let direct = graph.call_traces(&roots, CallTraversal::Direct, None);
    let duplicate_targets = direct
        .iter()
        .filter_map(|trace| match &trace.target {
            NodeId::Symbol {
                file: target_file,
                symbol,
                callable_id: Some(id),
            } if target_file.as_ref() == file.as_path() && symbol.as_ref() == "run/target" => {
                Some(*id)
            }
            _ => None,
        })
        .collect::<HashSet<_>>();
    assert_eq!(duplicate_targets.len(), 2);

    let transitive = graph.call_traces(&roots, CallTraversal::Transitive, None);
    for target in ["firstLeaf", "secondLeaf"] {
        assert!(
            transitive
                .iter()
                .any(|trace| { has_symbol(&trace.target, &file, target) })
        );
    }
}

#[test]
fn construction_traverses_the_constructor_without_promoting_other_methods() {
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
    let file = root.join("src/constructed-call-traversal.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: file.clone(),
        symbol: "run".to_string(),
    }]);
    let transitive = graph.call_traces(&roots, CallTraversal::Transitive, None);

    for target in ["Service", "Service/constructor", "constructorLeaf"] {
        assert!(
            transitive
                .iter()
                .any(|trace| { has_symbol(&trace.target, &file, target) }),
            "{target} must be reachable through new Service()"
        );
    }
    for target in ["Service/unused", "unusedLeaf"] {
        assert!(
            !transitive
                .iter()
                .any(|trace| { has_symbol(&trace.target, &file, target) }),
            "{target} is a non-invoked member"
        );
    }
}
