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
    let marked: Vec<_> = [
        "CleanClass",
        "UsedClass",
        "Built",
        "Local",
        "Deep",
        "C",
        "AliasedClass",
        "ArgumentClass",
        "ComputedClass",
        "LostClass",
        "MixedClass",
        "DefaultedClass",
        "LegacyClass",
        "CopiedClass",
        "WholeClass",
        "DynamicClass",
        "RequiredClass",
    ]
    .into_iter()
    .map(|scope| {
        let class = graph
            .class_declarations()
            .iter()
            .find(|class| class.scope == scope)
            .unwrap_or_else(|| panic!("no class {scope}"));
        (scope, class.namespace_escaped)
    })
    .collect();
    let escaped: Vec<_> = marked.iter().filter(|(_, e)| *e).map(|(s, _)| *s).collect();
    let resolved: Vec<_> = marked
        .iter()
        .filter(|(_, e)| !*e)
        .map(|(s, _)| *s)
        .collect();
    assert_eq!(
        resolved,
        ["CleanClass", "UsedClass", "Built", "Local", "Deep", "C"]
    );
    assert_eq!(
        escaped,
        [
            // Used in the file that declares it.
            "AliasedClass",
            "ArgumentClass",
            "ComputedClass",
            "LostClass",
            "MixedClass",
            "DefaultedClass",
            "LegacyClass",
            // Used from another file.
            "CopiedClass",
            "WholeClass",
            "DynamicClass",
            "RequiredClass",
        ]
    );
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
