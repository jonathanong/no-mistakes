use super::namespace_members::{build, callers, class, owned};

/// An `export *` source that exports a value, or a package the graph cannot
/// read, may supply the same name as a namespace from another `export *`
/// source. The barrel's name is then ambiguous: a construction through it is no
/// call edge to the namespace's class, and the namespace escapes.
#[test]
fn a_namespace_that_collides_with_another_star_export_is_not_followed() {
    let (root, graph) = build();
    for (scope, namespace) in [("ClashClass", "Clash"), ("AbroadClass", "Abroad")] {
        let declared = class(&graph, scope, Some(namespace));
        assert_eq!(callers(&root, &graph, declared), owned(&[]), "{scope}");
        assert!(declared.namespace_escaped, "{scope}");
    }
}
