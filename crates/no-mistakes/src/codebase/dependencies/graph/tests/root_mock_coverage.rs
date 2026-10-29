use super::super::*;
use crate::codebase::dependencies::graph::test_support;
use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
use crate::fx::fx_map;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn facts(path: &Path) -> TsFactMap {
    let mut facts = TsFactMap::new();
    facts.insert(
        path.to_path_buf(),
        TsFileFacts {
            module_bindings: Some(Arc::new(
                crate::codebase::ts_source::module_facts::TypeScriptModuleFacts::default(),
            )),
            ..TsFileFacts::default()
        },
    );
    facts
}

fn mock_graph(forward: EdgeMap) -> DepGraph {
    test_support::from_typed_maps(PathBuf::from("/repo"), forward, fx_map())
}

#[test]
fn automock_initializer_lookup_fails_closed_for_missing_and_non_file_facts() {
    let target = Path::new("/repo/target.ts");
    let empty = TsFactMap::new();
    let graph = mock_graph(fx_map());
    assert_eq!(
        graph.automock_initializer_seeds(target, &empty),
        (false, vec![])
    );

    let mut edges = fx_map();
    edges.insert(
        NodeId::file(target),
        vec![(NodeId::module("external"), EdgeKind::Import)],
    );
    let graph = mock_graph(edges);
    assert_eq!(
        graph.automock_initializer_seeds(target, &facts(target)),
        (false, vec![])
    );
}

#[test]
fn automock_initializer_fails_closed_for_incomplete_module_facts() {
    let target = Path::new("/repo/target.ts");
    for incomplete in [
        TsFileFacts {
            parse_error: Some("parse failed".into()),
            ..TsFileFacts::default()
        },
        TsFileFacts {
            module_bindings: Some(Arc::new(
                crate::codebase::ts_source::module_facts::TypeScriptModuleFacts::default(),
            )),
            operational_error: Some("fact collection failed".into()),
            ..TsFileFacts::default()
        },
        TsFileFacts::default(),
    ] {
        let mut prepared = facts(target);
        prepared.insert(target.to_path_buf(), incomplete);
        assert_eq!(
            mock_graph(fx_map()).automock_initializer_seeds(target, &prepared),
            (false, vec![])
        );
    }
}

#[test]
fn automock_initializer_keeps_module_scope_callers_as_seeds() {
    let target = Path::new("/repo/target.ts");
    let mut graph = mock_graph(fx_map());
    graph.resolved_call_sites.push(ResolvedCallSite {
        file: target.to_path_buf(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        source_callee: "initialize".into(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    });
    graph.call_sites_by_file.insert(target.to_path_buf(), 0..1);
    assert_eq!(
        graph.automock_initializer_seeds(target, &facts(target)),
        (false, vec![NodeId::file(target)])
    );
}

#[test]
fn mock_import_reachability_terminates_cycles_and_ignores_calls() {
    let a = NodeId::file("/repo/a.ts");
    let b = NodeId::file("/repo/b.ts");
    let mut edges = fx_map();
    edges.insert(a.clone(), vec![(b.clone(), EdgeKind::Import)]);
    edges.insert(b.clone(), vec![(a.clone(), EdgeKind::Import)]);
    assert!(
        !mock_graph(edges).import_reaches_mock(Path::new("/repo/a.ts"), Path::new("/repo/c.ts"))
    );

    let mut edges = fx_map();
    edges.insert(a, vec![(b, EdgeKind::Call)]);
    assert!(
        !mock_graph(edges).import_reaches_mock(Path::new("/repo/a.ts"), Path::new("/repo/b.ts"))
    );
}

#[test]
fn mock_factory_rejects_non_file_sources_and_missing_facts() {
    let module = NodeId::module("external");
    let graph = mock_graph(fx_map());
    assert!(!graph.mock_factory_is_full(
        &[module],
        Path::new("/repo/target.ts"),
        &TsFactMap::new()
    ));

    let factory = NodeId::file("/repo/factory.ts");
    assert!(!graph.mock_factory_is_full(
        &[factory],
        Path::new("/repo/target.ts"),
        &TsFactMap::new()
    ));

    for incomplete in [
        TsFileFacts {
            parse_error: Some("parse failed".into()),
            ..TsFileFacts::default()
        },
        TsFileFacts {
            module_bindings: Some(Arc::new(
                crate::codebase::ts_source::module_facts::TypeScriptModuleFacts::default(),
            )),
            operational_error: Some("fact collection failed".into()),
            ..TsFileFacts::default()
        },
        TsFileFacts::default(),
    ] {
        let mut prepared = TsFactMap::new();
        prepared.insert(PathBuf::from("/repo/factory.ts"), incomplete);
        assert!(!graph.mock_factory_is_full(
            &[NodeId::file("/repo/factory.ts")],
            Path::new("/repo/target.ts"),
            &prepared
        ));
    }
}

#[test]
fn root_mock_bypasses_include_actual_loaders_eval_and_function() {
    let site = |source_callee: &str| ResolvedCallSite {
        file: PathBuf::from("/repo/test.ts"),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        source_callee: source_callee.into(),
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    for name in ["vi.importActual", "jest.requireActual", "vi.importOriginal"] {
        assert!(root_mock_actual_bypass(&site(name)));
    }
    for name in ["eval", "Function"] {
        assert!(root_mock_dynamic_bypass(&site(name)));
    }
    let mut global = site("invoke");
    global.target = ResolvedCallTarget::Global {
        name: "eval".into(),
    };
    assert!(root_mock_dynamic_bypass(&global));
}

#[test]
fn bounded_mock_walk_honors_a_one_node_depth_cap() {
    let root = Path::new("/repo/root.ts");
    let child = NodeId::file("/repo/child.ts");
    let mut edges = fx_map();
    edges.insert(NodeId::file(root), vec![(child, EdgeKind::Call)]);
    let mut graph = mock_graph(edges);
    graph.root_mock_boundaries.insert(
        root.to_path_buf(),
        RootMockBoundaries {
            blocked: [PathBuf::from("/repo/blocked.ts")].into_iter().collect(),
            factories: Vec::new(),
        },
    );
    let selected = graph.call_source_nodes_with_root_mocks(&[NodeId::file(root)], Some(1));
    assert_eq!(selected, [NodeId::file(root)].into_iter().collect());
}
