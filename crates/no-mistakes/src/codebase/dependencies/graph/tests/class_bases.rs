use super::*;
use crate::codebase::dependencies::extract::{CallTargetIdentity, CallableId};

fn build(extends: bool, calls: bool) -> (PathBuf, DepGraph) {
    build_fixture("class-bases", extends, calls)
}

fn build_fixture(name: &str, extends: bool, calls: bool) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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
            extends,
            calls,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn relative<'a>(root: &Path, path: &'a Path) -> &'a str {
    path.strip_prefix(root).unwrap().to_str().unwrap()
}

/// Every out-edge of the class named `scope`, as `(kind, file, symbol)`.
fn out_edges(root: &Path, graph: &DepGraph, scope: &str) -> Vec<(EdgeKind, String, String)> {
    let class = graph
        .class_declarations()
        .iter()
        .find(|class| class.scope == scope)
        .unwrap();
    graph
        .dependencies_of_node(&class.node())
        .into_iter()
        .flatten()
        .map(|(target, kind)| match target {
            NodeId::Symbol { file, symbol, .. } => {
                (*kind, relative(root, file).to_string(), symbol.to_string())
            }
            other => panic!("expected a symbol target, got {other:?}"),
        })
        .collect()
}

#[test]
fn class_declarations_record_line_export_state_and_global_base() {
    let (root, graph) = build(true, false);
    let summary: Vec<_> = graph
        .class_declarations()
        .iter()
        .map(|class| {
            (
                relative(&root, &class.file),
                class.scope.as_str(),
                class.exported,
                class.line,
                class.global_base.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("packages/lib/index.ts", "LibError", true, 1, Some("Error")),
            ("src/classes.ts", "Local", true, 6, None),
            ("src/classes.ts", "FromNamespace", true, 7, None),
            ("src/classes.ts", "FromWorkspace", true, 8, None),
            ("src/classes.ts", "SameFile", true, 9, None),
            ("src/classes.ts", "Builtin", true, 10, Some("TypeError")),
            ("src/classes.ts", "FromPackage", true, 11, None),
            ("src/classes.ts", "Private", true, 15, Some("Error")),
            ("src/classes.ts", "default", true, 18, Some("Error")),
        ]
    );
}

/// A class inside any `namespace`, dotted namespace, `declare namespace`,
/// `declare module 'x'` or `declare global` block, or declared with `declare`,
/// is marked. The classes on both sides of the blocks are not: a depth counter
/// that missed a decrement would mark `After`.
#[test]
fn classes_in_module_blocks_or_declared_are_marked() {
    let (_, graph) = build_fixture("class-blocks", true, false);
    let marked: Vec<_> = graph
        .class_declarations()
        .iter()
        .map(|class| (class.scope.as_str(), class.namespaced_or_ambient))
        .collect();
    assert_eq!(
        marked,
        [
            ("Before", false),
            ("InNamespace", true),
            ("InNested", true),
            ("InDotted", true),
            ("InDeclareNamespace", true),
            ("InModule", true),
            ("InGlobal", true),
            ("Declared", true),
            ("After", false),
        ]
    );
}

#[test]
fn repository_bases_are_extends_edges_and_nothing_else() {
    let (root, graph) = build(true, false);
    let extends =
        |file: &str, symbol: &str| vec![(EdgeKind::Extends, file.to_string(), symbol.to_string())];
    assert_eq!(
        out_edges(&root, &graph, "Local"),
        extends("src/base.ts", "Base")
    );
    assert_eq!(
        out_edges(&root, &graph, "FromNamespace"),
        extends("src/base.ts", "Base")
    );
    // A workspace package name reaches the package source.
    assert_eq!(
        out_edges(&root, &graph, "FromWorkspace"),
        extends("packages/lib/index.ts", "LibError")
    );
    assert_eq!(
        out_edges(&root, &graph, "SameFile"),
        extends("src/classes.ts", "Local")
    );
    // Globals and unresolved packages have no node to point at.
    for scope in ["Builtin", "FromPackage", "Private", "default", "LibError"] {
        assert_eq!(out_edges(&root, &graph, scope), [], "{scope}");
    }
    assert!(graph
        .edges
        .edges()
        .iter()
        .all(|edge| edge.kind == EdgeKind::Extends));
}

#[test]
fn extends_alone_builds_no_call_surface() {
    let (_, graph) = build(true, false);
    assert!(graph.resolved_call_sites().is_empty());
    assert!(graph.callable_export_resolutions.is_empty());
}

#[test]
fn extends_is_purely_additive_to_the_call_graph() {
    let (root, with) = build(true, true);
    let (_, without) = build(false, true);
    let calls = |graph: &DepGraph| {
        graph
            .edges
            .edges()
            .iter()
            .filter(|edge| edge.kind != EdgeKind::Extends)
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(calls(&with), calls(&without));
    assert_eq!(with.resolved_call_sites(), without.resolved_call_sites());
    let exports = |graph: &DepGraph| {
        let mut keys: Vec<_> = graph.callable_export_resolutions.keys().cloned().collect();
        keys.sort();
        keys
    };
    assert_eq!(exports(&with), exports(&without));

    // A base is never a call: no site names one and no `Call` edge reaches a
    // class only because another class extends it.
    let classes = root.join("src/classes.ts");
    assert!(with
        .resolved_call_sites()
        .iter()
        .all(|site| site.file != classes || site.invocation != InvocationKind::Construct));
    let call_edges: Vec<_> = with
        .edges
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Call)
        .map(|edge| {
            let file = |node: &NodeId| match node {
                NodeId::File(path) | NodeId::Symbol { file: path, .. } => {
                    relative(&root, path).to_string()
                }
                other => panic!("unexpected node {other:?}"),
            };
            (file(&edge.from), file(&edge.to))
        })
        .collect();
    // Only `new LibError(...)` in `src/use.ts` is a call, seen from the file
    // and from `failure`; nothing in `src/classes.ts` calls anything.
    let use_to_lib = (
        "src/use.ts".to_string(),
        "packages/lib/index.ts".to_string(),
    );
    assert_eq!(call_edges, [use_to_lib.clone(), use_to_lib]);
}

#[test]
fn class_declarations_and_extends_edges_are_empty_unless_requested() {
    for calls in [false, true] {
        let (_, graph) = build(false, calls);
        assert!(graph.class_declarations().is_empty());
        assert!(graph
            .edges
            .edges()
            .iter()
            .all(|edge| edge.kind != EdgeKind::Extends));
    }
}

fn call(is_callback: bool, invocation: InvocationKind, owned: bool) -> FunctionCall {
    FunctionCall {
        caller: owned.then(|| "Child".to_string()),
        caller_id: owned.then_some(CallableId(7)),
        syntactic_caller: None,
        callee: "Base".to_string(),
        line: 1,
        offset: 0,
        is_callback,
        invocation,
        target_identity: CallTargetIdentity::Unknown,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn only_owned_callback_constructs_are_class_bases() {
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Construct, true)),
        Some((CallableId(7), "Child"))
    );
    assert_eq!(
        class_base_owner(&call(false, InvocationKind::Construct, true)),
        None
    );
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Call, true)),
        None
    );
    assert_eq!(
        class_base_owner(&call(true, InvocationKind::Construct, false)),
        None
    );
}

/// A call's caller is a callable node only when the extractor identified it.
#[test]
fn a_caller_node_carries_its_callable_id_only_when_identified() {
    let interner = crate::codebase::analysis_session::PathInterner::new();
    let file = Path::new("/repo/src/file.ts");
    let unidentified = NodeId::scoped_in(&interner, file, "Boom", None);
    assert!(matches!(
        unidentified,
        NodeId::Symbol {
            callable_id: None,
            ..
        }
    ));
    let identified = NodeId::scoped_in(&interner, file, "Boom", Some(CallableId(3)));
    assert!(matches!(
        identified,
        NodeId::Symbol {
            callable_id: Some(CallableId(3)),
            ..
        }
    ));
    assert_eq!(unidentified.as_symbol(), identified.as_symbol());
}
