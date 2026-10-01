use super::namespace_facts::{namespaces, uses};
use super::*;

fn locals(facts: &NamespaceFacts) -> Vec<(&str, &str)> {
    facts
        .locals
        .iter()
        .map(|(path, name)| (path.as_str(), name.as_str()))
        .collect()
}

/// The values a namespace body declares, whether or not they are exported,
/// each under the path of the body. Types are erased and declare no value, and
/// a declaration inside a function or class belongs to that scope, not the body.
#[test]
fn a_namespace_body_records_the_values_it_declares() {
    for (source, declared) in [
        ("namespace N { class A {} }", vec![("N", "A")]),
        ("namespace N { export class A {} }", vec![("N", "A")]),
        ("namespace N { function f() {} }", vec![("N", "f")]),
        ("namespace N { export function f() {} }", vec![("N", "f")]),
        ("namespace N { enum E { A } }", vec![("N", "E")]),
        ("namespace N { export enum E { A } }", vec![("N", "E")]),
        ("namespace N { namespace M {} }", vec![("N", "M")]),
        ("namespace N { export namespace M {} }", vec![("N", "M")]),
        ("namespace N { declare namespace M {} }", vec![("N", "M")]),
        (
            "namespace N { const a = 1, { b, c: [d] } = make(); }",
            vec![("N", "a"), ("N", "b"), ("N", "d")],
        ),
        ("namespace N { export let a = 1; }", vec![("N", "a")]),
        ("namespace N { var a = 1; }", vec![("N", "a")]),
        ("namespace N { import A = X.Y; }", vec![("N", "A")]),
        ("namespace N { export import A = X.Y; }", vec![("N", "A")]),
        (
            "namespace N.M { class A {} }",
            vec![("N", "M"), ("N.M", "A")],
        ),
        (
            "namespace N { interface I {} type T = number; export interface J {} }",
            vec![],
        ),
        ("namespace N { new A(); }", vec![]),
        (
            "namespace N { function f() { class Inner {} } }",
            vec![("N", "f")],
        ),
    ] {
        assert_eq!(locals(&namespaces(source)), declared, "{source}");
    }
}

/// A namespace declared in several blocks records only what every block
/// shares: a block's unexported declarations are private to it, and the facts
/// name a namespace by its path alone. Its exported members are shared when its
/// blocks are one namespace: top-level blocks always are, and nested blocks are
/// when each is exported from blocks that are. A dotted `namespace N.M` exports
/// `M` from `N`.
#[test]
fn a_namespace_declared_in_several_blocks_records_only_what_they_share() {
    for (source, declared) in [
        (
            "namespace N { class A {} }\nnamespace N { class B {} }",
            vec![],
        ),
        (
            "namespace N { export class A {} }\nnamespace N { export const b = 1; }",
            vec![("N", "A"), ("N", "b")],
        ),
        (
            "namespace N { class A {} }\nnamespace N { export namespace M { class B {} } }",
            vec![("N", "M"), ("N.M", "B")],
        ),
        (
            "namespace N { namespace M { class A {} } }\nnamespace N { namespace M { class B {} } }",
            vec![],
        ),
        (
            "namespace N { export namespace M { export class A {} } }\nnamespace N { export namespace M { class B {} } }",
            vec![("N", "M"), ("N.M", "A")],
        ),
        (
            "namespace N { namespace M { export class A {} } }\nnamespace N { namespace M {} }",
            vec![],
        ),
        (
            "namespace N { class A {} }\nnamespace O { class B {} }\nnamespace O { class C {} }",
            vec![("N", "A")],
        ),
        (
            "namespace N.M { class A {} }\nnamespace N { class B {} }",
            vec![("N", "M"), ("N.M", "A")],
        ),
        (
            "namespace N.M { class A {} }\nnamespace O.P { class B {} }",
            vec![("N", "M"), ("N.M", "A"), ("O", "P"), ("O.P", "B")],
        ),
    ] {
        assert_eq!(locals(&namespaces(source)), declared, "{source}");
    }
}

/// An import read through a string literal is followed like a dot, but only
/// directly off the import: after another member the graph cannot name the
/// class, so the construction reads the import's member (`E.Inner`) as a value,
/// which the graph escapes. A computed member is a use of the whole import.
#[test]
fn a_string_literal_member_after_another_member_reads_the_import_as_a_value() {
    for (source, used, read) in [
        ("new E.Dead();", vec![], vec![]),
        ("new E['Dead']();", vec![], vec![]),
        ("new E['Inner'].Dead();", vec![], vec![]),
        ("new E.Inner.Dead();", vec![], vec![]),
        ("new E.Inner['Dead']();", vec![], vec![("E", "Inner")]),
        ("new E[name]();", vec!["E"], vec![]),
    ] {
        let facts = namespaces(&format!("import {{ E }} from './m';\n{source}"));
        let reads: Vec<_> = facts
            .member_uses
            .iter()
            .map(|(local, member)| (local.as_str(), member.as_str()))
            .collect();
        assert_eq!(uses(&facts), used, "{source}");
        assert_eq!(reads, read, "{source}");
    }
}
