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
            "SelfBuilt",
            "Errors",
            owned(&[("src/errors.ts", Some("SelfBuilt/create"))]),
        ),
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

/// A parameter named like the namespace, or like one of its classes, shadows
/// it: the construction builds no class of the namespace, and is no escape.
#[test]
fn a_construction_through_a_shadowing_binding_is_not_a_namespace_member() {
    let (root, graph) = build();
    for scope in ["ShadowDead", "InnerShadow"] {
        let shadowed = class(&graph, scope, Some("Shadowed"));
        assert!(callers(&root, &graph, shadowed).is_empty(), "{scope}");
        assert!(!shadowed.namespace_escaped, "{scope}");
    }
}

/// Whether the namespace of the class `scope` declared in `namespace` escaped.
fn escaped_in(graph: &DepGraph, scope: &str, namespace: &str) -> bool {
    class(graph, scope, Some(namespace)).namespace_escaped
}

/// A constant named like a nested namespace and a parameter named like a root
/// are not those namespaces; a bare name in the body that does denote the
/// nested namespace is a use of it.
#[test]
fn a_value_that_only_shares_a_name_with_a_namespace_is_no_use_of_it() {
    let (_, graph) = build();
    assert!(!escaped_in(&graph, "CollideDead", "Collide.Inner"));
    assert!(!escaped_in(&graph, "HideDead", "Hide"));
    assert!(escaped_in(&graph, "ReachClass", "Reach.Inner"));
}

/// `typeof Ns`, `implements Ns.I` and `interface X extends Ns.I` name the
/// namespace in erased code, so none of them is a use at run time.
#[test]
fn a_namespace_named_only_in_erased_types_is_not_escaped() {
    let (_, graph) = build();
    for (scope, namespace) in [
        ("QueriedClass", "Queried"),
        ("MarkedClass", "Marked"),
        ("ExtendedClass", "Extended"),
    ] {
        assert!(!escaped_in(&graph, scope, namespace), "{namespace}");
    }
}

/// `Ns.X.bind(..)`, `.call(..)` and `.apply(..)` hand the class on as a value,
/// in the file and through an import; a static guard `Ns.X.is(..)` does not.
#[test]
fn a_member_handed_on_by_bind_call_or_apply_escapes_its_namespace() {
    let (_, graph) = build();
    for (scope, namespace) in [
        ("BoundClass", "Bound"),
        ("CalledClass", "Called"),
        ("AppliedClass", "Applied"),
        ("HandedClass", "Handed"),
    ] {
        assert!(escaped_in(&graph, scope, namespace), "{namespace}");
    }
    assert!(!escaped_in(&graph, "GuardedClass", "Guarded"));
}

/// A module read whole exposes what it exports: a namespace its barrel
/// re-exports by name escapes, and one its source keeps to itself does not.
#[test]
fn a_barrel_read_whole_exposes_only_the_namespaces_it_re_exports() {
    let (_, graph) = build();
    assert!(escaped_in(&graph, "ExposedClass", "Exposed"));
    assert!(!escaped_in(&graph, "KeptClass", "Kept"));
}

/// `export { Hidden as Public } from "./m"` exports the namespace of `m`, not
/// the local namespace of the same name, which this file never exports.
#[test]
fn a_sourced_export_clause_does_not_export_a_local_namespace() {
    let (root, graph) = build();
    let remote = class(&graph, "RemoteLive", Some("Hidden"));
    assert_eq!(
        callers(&root, &graph, remote),
        owned(&[("src/scopes/sourced-use.ts", Some("live"))])
    );
    assert!(!remote.namespace_escaped);
    assert!(!class(&graph, "LocalHidden", Some("Hidden")).exported);
}

/// An import of the namespace's name merges with it, so it escapes; an erased
/// `import type x = require()` is no use of the module at all.
#[test]
fn an_import_merges_with_a_namespace_but_an_erased_one_uses_nothing() {
    let (_, graph) = build();
    assert!(class(&graph, "ImportMergedClass", Some("Imported")).namespace_escaped);
    assert!(!class(&graph, "TypeOnlyClass", Some("TypeOnly")).namespace_escaped);
}

/// A local declared in one namespace's body, `const` or hoisted `var`, is bound
/// only there, so the construction written after it still names the imported
/// namespace.
#[test]
fn a_local_in_a_namespace_body_does_not_shadow_an_import_outside_it() {
    let (root, graph) = build();
    for (scope, namespace, dead, caller) in [
        ("Built", "BodyErrors", "BodyDead", "built"),
        ("VarBuilt", "VarErrors", "VarDead", "varBuilt"),
    ] {
        let built = class(&graph, scope, Some(namespace));
        assert_eq!(
            callers(&root, &graph, built),
            owned(&[
                ("src/scopes/body-scope.ts", None),
                ("src/scopes/body-scope.ts", Some(caller)),
            ])
        );
        assert!(!built.namespace_escaped);
        assert!(!escaped_in(&graph, dead, namespace));
    }
}

/// A local of a namespace body that shares a declared namespace's name is that
/// local inside the body: the `new` there reaches no class of the namespace.
#[test]
fn a_local_in_a_namespace_body_hides_a_declared_namespace_of_its_name() {
    let (root, graph) = build();
    let dead = class(&graph, "ShadowDead", Some("ShadowErrors"));
    assert_eq!(callers(&root, &graph, dead), owned(&[]));
    assert!(!dead.namespace_escaped);
}

/// An import read through one static member (`target.version`, or a string
/// literal for the same member) is a use of that export alone: it escapes the
/// namespace the member names, and no other.
#[test]
fn a_member_read_through_a_namespace_import_escapes_only_that_export() {
    let (_, graph) = build();
    assert!(!escaped_in(&graph, "ReadKeptDead", "ReadKept"));
    assert!(escaped_in(&graph, "ReadHandedDead", "ReadHanded"));
    assert!(escaped_in(&graph, "ReadLiteralDead", "ReadLiteral"));
}

/// A class named bare in the body that declares it, or in a namespace nested
/// there, is a value like `Errors.Dead` would be: handing it on, or binding it,
/// escapes the namespace. A parameter of the class's name, a guard, and a type
/// hand nothing on.
#[test]
fn a_class_named_bare_in_its_namespace_body_escapes_when_read_as_a_value() {
    let (_, graph) = build();
    assert!(escaped_in(&graph, "BareHandedDead", "BareHanded"));
    assert!(escaped_in(&graph, "BareNestedDead", "BareNested"));
    assert!(escaped_in(&graph, "BareBoundDead", "BareBound"));
    assert!(!escaped_in(&graph, "BareKeptDead", "BareKept"));
}

/// A bare decorator is a call of the member it names, so it is not a value use
/// of its namespace; the dead class beside it stays unescaped.
#[test]
fn a_bare_decorator_does_not_escape_its_namespace() {
    let (_, graph) = build();
    assert!(!escaped_in(&graph, "DecoratedDead", "Decorated"));
}

/// A class and a namespace of one name in one body are a single value, so
/// reading it hands on the namespace's classes too.
#[test]
fn a_class_merged_with_a_namespace_in_one_body_escapes_when_read() {
    let (_, graph) = build();
    assert!(escaped_in(&graph, "Deep", "MergedBody.Inner"));
}

/// A graph built for `extends` alone has no call edge, but its class
/// declarations still say which namespaces a construction could not follow.
#[test]
fn an_extends_only_graph_marks_the_same_namespaces_escaped() {
    let (_, full) = build();
    let (_, extends_only) = build_fixture("namespace-members", true, false);
    let flags = |graph: &DepGraph| {
        let mut flags: Vec<_> = graph
            .class_declarations()
            .iter()
            .map(|class| {
                (
                    class.file.clone(),
                    class.scope.clone(),
                    class.namespace_escaped,
                )
            })
            .collect();
        flags.sort();
        flags
    };
    assert_eq!(flags(&extends_only), flags(&full));
    assert!(class(&extends_only, "LostClass", Some("Lost")).namespace_escaped);
    assert!(extends_only
        .edges
        .edges()
        .iter()
        .all(|edge| edge.kind != EdgeKind::Call));
}
