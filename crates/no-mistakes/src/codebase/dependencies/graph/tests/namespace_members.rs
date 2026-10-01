use super::class_bases::{build_fixture, relative};
use super::*;
use crate::codebase::dependencies::extract::CallableId;

fn build() -> (PathBuf, DepGraph) {
    build_fixture("namespace-members", true, true)
}

/// The class `scope` declared in `namespace`, or at the top level for `None`.
fn class<'a>(graph: &'a DepGraph, scope: &str, namespace: Option<&str>) -> &'a ClassDeclaration {
    graph
        .class_declarations()
        .iter()
        .find(|class| class.scope == scope && class.namespace.as_deref() == namespace)
        .unwrap_or_else(|| panic!("no class {namespace:?} {scope}"))
}

/// Where each `Call` edge into `class` starts, as `(file, caller scope)`.
fn callers(
    root: &Path,
    graph: &DepGraph,
    class: &ClassDeclaration,
) -> Vec<(String, Option<String>)> {
    let target = class.node();
    let mut callers: Vec<_> = graph
        .edges
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Call && edge.to == target)
        .map(|edge| match &edge.from {
            NodeId::File(file) => (relative(root, file).to_string(), None),
            NodeId::Symbol { file, symbol, .. } => {
                (relative(root, file).to_string(), Some(symbol.to_string()))
            }
            other => panic!("unexpected caller {other:?}"),
        })
        .collect();
    callers.sort();
    callers.dedup();
    callers
}

/// The ids of the classes `class` extends, by its `Extends` edges.
fn bases(graph: &DepGraph, class: &ClassDeclaration) -> Vec<Option<CallableId>> {
    let source = class.node();
    graph
        .edges
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Extends && edge.from == source)
        .map(|edge| match &edge.to {
            NodeId::Symbol { callable_id, .. } => *callable_id,
            other => panic!("expected a symbol, got {other:?}"),
        })
        .collect()
}

fn owned(callers: &[(&str, Option<&str>)]) -> Vec<(String, Option<String>)> {
    callers
        .iter()
        .map(|(file, caller)| (file.to_string(), caller.map(str::to_string)))
        .collect()
}

/// A `new` that names a namespace member is a call edge to that class, whether
/// the path is qualified, bare inside the namespace body, nested, dotted, or
/// written through an import. A base class is never a call. A top-level
/// `export const x = new ...` is a call from the file and from the exported `x`.
#[test]
fn a_namespace_member_construction_is_a_call_edge_to_the_class() {
    let (root, graph) = build();
    let cases = [
        (
            "Built",
            "Errors",
            owned(&[
                ("src/consumer.ts", Some("viaModule")),
                ("src/errors.ts", None),
                ("src/errors.ts", Some("built")),
            ]),
        ),
        ("Local", "Errors", owned(&[("src/errors.ts", Some("make"))])),
        (
            "Deep",
            "Errors.Inner",
            owned(&[
                ("src/consumer.ts", Some("viaRename")),
                ("src/errors.ts", None),
                ("src/errors.ts", Some("deep")),
            ]),
        ),
        (
            "C",
            "A.B",
            owned(&[
                ("src/consumer.ts", Some("viaDotted")),
                ("src/errors.ts", None),
                ("src/errors.ts", Some("dotted")),
            ]),
        ),
        // Two classes of one name: each is built only from its own namespace.
        ("Twin", "One", owned(&[("src/errors.ts", Some("one"))])),
        ("Twin", "Two", owned(&[])),
        ("Base", "Errors", owned(&[])),
    ];
    for (scope, namespace, expected) in cases {
        let found = callers(&root, &graph, class(&graph, scope, Some(namespace)));
        assert_eq!(found, expected, "{namespace}.{scope}");
    }
}

/// A namespace is also reached through the ways a module passes one on: a
/// named or renamed re-export, an import that is exported again, a default
/// import exported under another name, and `export *`. A default export is not
/// carried by `export *`, and an export that is not a namespace reaches no class.
#[test]
fn a_namespace_reached_through_a_re_export_is_a_call_edge() {
    let (root, graph) = build();
    let cases = [
        (
            "HubClass",
            "Hub",
            owned(&[
                ("src/relay/user.ts", Some("named")),
                ("src/relay/user.ts", Some("repeated")),
                ("src/relay/user.ts", Some("viaStar")),
            ]),
        ),
        (
            "OtherClass",
            "Other",
            owned(&[("src/relay/user.ts", Some("renamed"))]),
        ),
        (
            "SpareClass",
            "Spare",
            owned(&[("src/relay/user.ts", Some("spare"))]),
        ),
        (
            "PivotClass",
            "Pivot",
            owned(&[
                ("src/relay/user.ts", Some("direct")),
                ("src/relay/user.ts", Some("hinge")),
            ]),
        ),
    ];
    for (scope, namespace, expected) in cases {
        let found = callers(&root, &graph, class(&graph, scope, Some(namespace)));
        assert_eq!(found, expected, "{namespace}.{scope}");
    }
}

/// A graph built for calls alone resolves `new Errors.X()` through an import
/// the same way: the edge does not depend on the class declarations that an
/// `extends` pass records.
#[test]
fn a_graph_built_for_calls_alone_still_resolves_namespace_members() {
    let (root, graph) = build_fixture("namespace-members", false, true);
    assert!(graph.class_declarations().is_empty());
    let targets: Vec<_> = graph
        .edges
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Call)
        .filter_map(|edge| match (&edge.from, &edge.to) {
            (
                NodeId::Symbol { file, symbol, .. },
                NodeId::Symbol {
                    file: target,
                    symbol: class,
                    ..
                },
            ) if relative(&root, file) == "src/consumer.ts" && &**symbol == "viaModule" => {
                Some((relative(&root, target).to_string(), class.to_string()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        targets,
        [("src/errors.ts".to_string(), "Built".to_string())]
    );
}

/// An `extends` that names a namespace member reaches that class by its id,
/// from the namespace body, the top level, another file, and between two
/// same-named classes of different namespaces.
#[test]
fn a_namespace_member_base_is_an_extends_edge_to_the_exact_class() {
    let (_, graph) = build();
    let id = |scope: &str, namespace: &str| Some(class(&graph, scope, Some(namespace)).callable_id);
    let cases = [
        (class(&graph, "Child", Some("Errors")), id("Base", "Errors")),
        (class(&graph, "TopLevel", None), id("Base", "Errors")),
        (class(&graph, "Sub", None), id("Base", "Errors")),
        (class(&graph, "TwinChild", Some("Two")), id("Twin", "Two")),
    ];
    for (child, base) in cases {
        assert_eq!(bases(&graph, child), [base], "{}", child.scope);
    }
    assert_ne!(id("Twin", "One"), id("Twin", "Two"));
}

/// `namespace_escaped` is set exactly when some use of the namespace is not a
/// static member access the graph resolves.
#[test]
fn a_namespace_is_marked_escaped_when_a_use_cannot_be_followed() {
    let (_, graph) = build();
    let escaped = |scope: &str| {
        graph
            .class_declarations()
            .iter()
            .find(|class| class.scope == scope)
            .unwrap_or_else(|| panic!("no class {scope}"))
            .namespace_escaped
    };
    let resolved = [
        "CleanClass",
        "UsedClass",
        "Built",
        "Local",
        "Deep",
        "C",
        // Reached through a re-export the graph follows.
        "HubClass",
        "OtherClass",
        "SpareClass",
        "PivotClass",
    ];
    let used_in_place = [
        "AliasedClass",
        "ArgumentClass",
        "ComputedClass",
        "LostClass",
        // `Gap.Missing.Factory`: `Gap` has no `Missing`.
        "GapClass",
        "MixedClass",
        "DefaultedClass",
        "LegacyClass",
    ];
    let used_elsewhere = [
        "CopiedClass",
        "WholeClass",
        "DynamicClass",
        "RequiredClass",
        // The chain of exports cannot be followed to a namespace.
        "StarredClass",
        "WrappedClass",
        "HauntedClass",
        "LoopAClass",
        "LoopBClass",
        "SharedOne",
        "SharedTwo",
        "NowhereClass",
        // An import read as a value.
        "FallbackClass",
        "CopiedStarClass",
    ];
    for scope in resolved {
        assert!(!escaped(scope), "{scope} should resolve");
    }
    for scope in used_in_place.into_iter().chain(used_elsewhere) {
        assert!(escaped(scope), "{scope} should escape");
    }
}

/// Only a namespace's classes are marked. A top-level class is never escaped,
/// and one in an ambient block is in no namespace.
#[test]
fn classes_outside_a_namespace_are_never_marked_escaped() {
    let (_, graph) = build();
    assert!(graph
        .class_declarations()
        .iter()
        .filter(|class| class.namespace.is_none())
        .all(|class| !class.namespace_escaped));
}
