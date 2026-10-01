use super::namespace_members::{bases, build, callers, class, escaped_in, owned};

const SHADOW: &str = "src/scopes/class-shadow.ts";

/// A class, function, enum, or `import` alias declared in a namespace body is a
/// name of that body, so it hides an import of the same name there, as a `var`
/// does. The constructions and the base in `scopes/class-shadow.ts` name those
/// values: none of them is a call or an `extends` edge into the imported
/// classes.
#[test]
fn a_declaration_in_a_namespace_body_hides_an_import_of_its_name() {
    let (root, graph) = build();
    for (dead, namespace) in [
        ("ClassDead", "ClassErrors"),
        ("FunctionDead", "FunctionErrors"),
        ("EnumDead", "EnumErrors"),
        ("LaterDead", "LaterErrors"),
        ("AliasDead", "AliasErrors"),
        ("ClassBase", "ClassErrors"),
    ] {
        let imported = class(&graph, dead, Some(namespace));
        assert_eq!(callers(&root, &graph, imported), owned(&[]), "{dead}");
        assert!(!imported.namespace_escaped, "{dead}");
    }
    let sub = class(&graph, "Sub", Some("ShadowBodies"));
    assert_eq!(bases(&graph, sub), vec![]);
}

/// A bare `new` is hidden the same way: the body's function of that name is
/// what it builds, not the imported class.
#[test]
fn a_function_in_a_namespace_body_hides_an_imported_class_of_its_name() {
    let (root, graph) = build();
    let imported = class(&graph, "BareImported", None);
    assert_eq!(callers(&root, &graph, imported), owned(&[]));
}

/// A namespace nested in the body that shares an import's name is what the
/// construction names: it builds the nested class and reaches nothing of the
/// import.
#[test]
fn a_namespace_in_a_namespace_body_hides_an_import_of_its_name() {
    let (root, graph) = build();
    let local = class(&graph, "NestedLocal", Some("ShadowBodies.NestedErrors"));
    assert_eq!(
        callers(&root, &graph, local),
        owned(&[(SHADOW, None), (SHADOW, Some("byNested"))])
    );
    let imported = class(&graph, "NestedDead", Some("NestedErrors"));
    assert_eq!(callers(&root, &graph, imported), owned(&[]));
    assert!(!escaped_in(&graph, "NestedDead", "NestedErrors"));
}

/// A class declared in one block of a merged namespace is private to that
/// block, so a construction in another block still names the import. So is an
/// unexported namespace: one in each block is two namespaces, and the second
/// cannot see the first one's member.
#[test]
fn a_declaration_in_one_block_of_a_merged_namespace_hides_nothing_in_another() {
    let (root, graph) = build();
    for (dead, namespace, caller) in [
        ("MergedDead", "MergedErrors", "second"),
        ("SplitDead", "SplitErrors", "split"),
    ] {
        let imported = class(&graph, dead, Some(namespace));
        assert_eq!(
            callers(&root, &graph, imported),
            owned(&[(SHADOW, None), (SHADOW, Some(caller))]),
            "{dead}"
        );
    }
}

/// An exported member is shared by every block of a merged namespace, one
/// level down too, and a dotted declaration exports its last name: another
/// block's construction builds that member's class, none of the import's.
#[test]
fn an_exported_member_of_a_merged_namespace_hides_an_import_in_every_block() {
    let (root, graph) = build();
    for (dead, local, caller) in [
        ("SharedDead", "SharedBlocks.SharedErrors", "shared"),
        ("DeepDead", "DeepBlocks.Inner.DeepErrors", "deep"),
        ("DottedDead", "DottedBlocks.DottedErrors", "dotted"),
    ] {
        let built = class(&graph, dead, Some(local));
        assert_eq!(
            callers(&root, &graph, built),
            owned(&[(SHADOW, None), (SHADOW, Some(caller))]),
            "{dead}"
        );
        // The import is the namespace of the local one's last name.
        let imported = class(&graph, dead, local.rsplit('.').next());
        assert_eq!(callers(&root, &graph, imported), owned(&[]), "{dead}");
    }
}
