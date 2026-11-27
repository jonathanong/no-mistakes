use super::namespace_facts::{namespaces, uses};
use super::*;

fn member_reads(facts: &NamespaceFacts) -> Vec<(&str, &str)> {
    facts
        .member_uses
        .iter()
        .map(|(local, member)| (local.as_str(), member.as_str()))
        .collect()
}

/// A name read through one static member (a dot, or a string literal) uses the
/// export that member names, not every export of the module, so the graph can
/// escape just that one. A bare, computed, wrapped or handed-on name still uses
/// the whole of it, and a declared namespace is escaped whole either way.
#[test]
fn a_name_read_through_a_static_member_is_a_use_of_that_member_alone() {
    for (source, read, whole) in [
        (
            "import * as mod from './m';\nconst v = mod.version;",
            vec![("mod", "version")],
            vec![],
        ),
        (
            "import * as mod from './m';\nconst v = mod['version'];",
            vec![("mod", "version")],
            vec![],
        ),
        (
            "import * as mod from './m';\nconst v = mod?.version;",
            vec![("mod", "version")],
            vec![],
        ),
        (
            "import * as mod from './m';\nregister(mod.Errors.Dead);",
            vec![("mod", "Errors")],
            vec![],
        ),
        (
            "import { Lib } from './lib';\nLib.A.bind(null);",
            vec![("Lib", "A")],
            vec![],
        ),
        (
            "import * as mod from './m';\nconst whole = mod;",
            vec![],
            vec!["mod"],
        ),
        (
            "import * as mod from './m';\nregister(mod);",
            vec![],
            vec!["mod"],
        ),
        (
            "import * as mod from './m';\nconst v = mod[name];",
            vec![],
            vec!["mod"],
        ),
        (
            "import * as mod from './m';\nconst v = (mod as any).version;",
            vec![],
            vec!["mod"],
        ),
        (
            "import * as mod from './m';\nfunction f(mod: any) { return mod.version; }",
            vec![],
            vec![],
        ),
        ("namespace N {}\nconst v = N.A;", vec![], vec!["N"]),
        ("namespace N {}\nconst v = N['A'];", vec![], vec!["N"]),
    ] {
        let facts = namespaces(source);
        assert_eq!(member_reads(&facts), read, "{source}");
        assert_eq!(uses(&facts), whole, "{source}");
    }
}

/// A class named bare inside the namespace that declares it, or one nested in
/// it, is a value of that namespace when read: handed on, bound, aliased. Built,
/// extended, guarded, typed, or shadowed by a local, it is no use.
#[test]
fn a_class_named_bare_in_its_namespace_body_is_a_use_of_that_namespace() {
    for (source, used) in [
        ("namespace N { export class A {} register(A); }", vec!["N"]),
        ("namespace N { class A {} const alias = A; }", vec!["N"]),
        (
            "namespace N { export class A {} export namespace M { register(A); } }",
            vec!["N"],
        ),
        (
            "namespace N.M { export class A {} register(A); }",
            vec!["N.M"],
        ),
        (
            "namespace N { export class A {} export const b = A.bind(null); }",
            vec!["N"],
        ),
        ("namespace N { export class A {} A.call(null); }", vec!["N"]),
        (
            "namespace N { export class A {} const p = A.prototype; }",
            vec!["N"],
        ),
        (
            "namespace N { export class A {} export function f() { return A; } }",
            vec!["N"],
        ),
        ("namespace N { export class A {} new A(); }", vec![]),
        ("namespace N { export class A {} A.is(error); }", vec![]),
        (
            "namespace N { export class A {} class B extends A {} }",
            vec![],
        ),
        (
            "namespace N { export class A {} error instanceof A; }",
            vec![],
        ),
        (
            "namespace N { export class A {} let a: A; type T = typeof A; }",
            vec![],
        ),
        (
            "namespace N { export class A {} function f(A: unknown) { register(A); } }",
            vec![],
        ),
        (
            "namespace N { export class A {} namespace M { const A = 1; register(A); } }",
            vec![],
        ),
        ("namespace N { export class A {} }\nregister(A);", vec![]),
        (
            "namespace N { export class A {} }\nnamespace M { register(A); }",
            vec![],
        ),
    ] {
        let facts = namespaces(source);
        assert_eq!(uses(&facts), used, "{source}");
    }
}
