use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;

fn namespaces(source: &str) -> NamespaceFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:#?}",
        parsed.diagnostics
    );
    extract_import_facts_from_program_with_source(&parsed.program, source).namespaces
}

fn members(facts: &NamespaceFacts) -> Vec<(&str, bool)> {
    let mut members: Vec<_> = facts
        .members
        .iter()
        .map(|member| (member.path.as_str(), member.exported))
        .collect();
    members.sort_unstable();
    members
}

fn uses(facts: &NamespaceFacts) -> Vec<&str> {
    facts.value_uses.iter().map(String::as_str).collect()
}

fn member_reads(facts: &NamespaceFacts) -> Vec<(&str, &str)> {
    facts
        .member_uses
        .iter()
        .map(|(local, member)| (local.as_str(), member.as_str()))
        .collect()
}

#[test]
fn members_carry_their_path_and_whether_another_module_can_reach_them() {
    let facts = namespaces(concat!(
        "export namespace Open {\n",
        "  export class A extends Error {}\n",
        "  class Hidden extends Error {}\n",
        "  export namespace Inner { export class B extends Error {} }\n",
        "  namespace Closed { export class C extends Error {} }\n",
        "}\n",
        "export namespace Dotted.Path { export class D extends Error {} }\n",
        "namespace Local { export class E extends Error {} }\n",
        "namespace Clause { export class F extends Error {} }\n",
        "export { Clause };\n",
        "export declare namespace Ambient { class G {} }\n",
        "declare namespace AmbientToo { class H {} }\n",
        "namespace Plain { declare class I {} export namespace Amb2 { class J {} } }\n",
    ));
    assert_eq!(
        members(&facts),
        [
            ("Clause.F", true),
            ("Dotted.Path.D", true),
            ("Local.E", false),
            ("Open.A", true),
            ("Open.Closed.C", false),
            ("Open.Hidden", false),
            ("Open.Inner.B", true),
            ("Plain.Amb2.J", false),
        ]
    );
    assert_eq!(
        facts.declared,
        [
            "Clause",
            "Dotted",
            "Dotted.Path",
            "Local",
            "Open",
            "Open.Closed",
            "Open.Inner",
            "Plain",
            "Plain.Amb2"
        ]
    );
}

#[test]
fn roots_list_the_names_the_file_exports_and_flag_merges() {
    let facts = namespaces(concat!(
        "export namespace Keyword { export class A {} }\n",
        "namespace Renamed { export class B {} }\n",
        "export { Renamed as Public, Renamed as default };\n",
        "export type { Renamed as OnlyType };\n",
        "namespace Private { export class C {} }\n",
        "export namespace Twice { export class D {} }\n",
        "export namespace Twice { export class E {} }\n",
        "export class WithClass {}\n",
        "export namespace WithClass { export class F {} }\n",
        "export enum WithEnum { One }\n",
        "export namespace WithEnum { export class G {} }\n",
        "function withFunction() {}\n",
        "namespace withFunction { export class H {} }\n",
    ));
    let roots: Vec<_> = facts
        .roots
        .iter()
        .map(|root| (root.name.as_str(), root.exports.as_slice(), root.merged))
        .collect();
    let public = ["Public".to_string(), "default".to_string()];
    assert_eq!(
        roots,
        [
            ("Keyword", &["Keyword".to_string()][..], false),
            ("Private", &[][..], false),
            ("Renamed", &public[..], false),
            ("Twice", &["Twice".to_string()][..], false),
            ("WithClass", &["WithClass".to_string()][..], true),
            ("WithEnum", &["WithEnum".to_string()][..], true),
            ("withFunction", &[][..], true),
        ]
    );
}

#[test]
fn only_the_uses_the_graph_resolves_leave_a_namespace_unescaped() {
    let facts = namespaces(concat!(
        "import { Lib } from './lib';\n",
        "namespace Errors { export class A extends Error {} export function make() {} }\n",
        "export { Errors };\n",
        "const built = new Errors.A();\n",
        "const called = Errors.make();\n",
        "const tagged = Errors.make`x`;\n",
        "const guarded = built instanceof Errors.A;\n",
        "class Child extends Errors.A {}\n",
        "const external = new Lib.Thing();\n",
        "const chained = (0, Errors.make)();\n",
        "const viaString = new Errors['A']();\n",
        // A type annotation names the class as a type, never as a value.
        "function typed(error: Errors.A): Errors.A { return error; }\n",
        // A bare decorator is an invocation of the member it names.
        "@Errors.make\nclass Decorated { @Errors.make method() {} @Errors.make field = 1; }\n",
    ));
    assert!(uses(&facts).is_empty(), "{:?}", facts.value_uses);
}

#[test]
fn a_namespace_that_is_read_as_a_value_is_recorded() {
    for (source, used) in [
        ("namespace N {}\nconst alias = N;", "N"),
        ("namespace N {}\nconst { A } = N;", "N"),
        ("namespace N {}\nregister(N);", "N"),
        ("namespace N {}\nregister(N.A);", "N"),
        ("namespace N {}\nconst x = N.A.CODE;", "N"),
        ("namespace N {}\nnew N[name]();", "N"),
        ("namespace N {}\nexport default N;", "N"),
        ("namespace N {}\nexport = N;", "N"),
        ("namespace N {}\nexport const o = { N };", "N"),
        (
            "namespace N { export namespace M {} }\nimport Alias = N.M;",
            "N",
        ),
        ("namespace N {}\nnew N.A.B['c']();", "N"),
        ("namespace N {}\nregister(class extends N.A {});", "N"),
        ("namespace N {}\nvalue instanceof N[kind];", "N"),
        ("namespace N {}\nnew (N as any).A();", ""),
        ("import { Lib } from './lib';\nconst alias = Lib;", "Lib"),
        ("import Lib from './lib';\nregister(Lib);", "Lib"),
        (
            "import * as lib from './lib';\nexport const whole = lib;",
            "lib",
        ),
    ] {
        let facts = namespaces(source);
        let expected: Vec<&str> = if used.is_empty() { vec![] } else { vec![used] };
        assert_eq!(uses(&facts), expected, "{source}");
    }
}

/// A value use names the namespace it resolves to: the nearest declared path,
/// looking outward from the body the use is in. A same-named value that is no
/// namespace, or a local that shadows the name, is no use of one.
#[test]
fn a_value_use_names_the_declared_namespace_it_resolves_to() {
    for (source, used) in [
        (
            "namespace A { export namespace Inner {} }\nconst Inner = {};\nregister(Inner);",
            vec![],
        ),
        (
            "namespace A { export namespace Inner {} export const x = Inner; }",
            vec!["A.Inner"],
        ),
        (
            "namespace B { export namespace Inner {} }\nnamespace A { export namespace Inner {} export const x = Inner; }",
            vec!["A.Inner"],
        ),
        (
            "namespace Inner {}\nnamespace A { export namespace Inner {} export const x = Inner; }",
            vec!["A.Inner"],
        ),
        (
            "namespace Inner {}\nnamespace A { export const x = Inner; }",
            vec!["Inner"],
        ),
        (
            "namespace A.B { export const x = B; }\nregister(B);",
            vec!["A.B"],
        ),
        ("namespace N {}\nfunction f(N: unknown) { register(N); }", vec![]),
        (
            "namespace N {}\nfunction f() { const N = 1; register(N); }",
            vec![],
        ),
        (
            "import { Lib } from './lib';\nfunction f(Lib: unknown) { register(Lib); }",
            vec![],
        ),
        // A local of a namespace body hides the name inside that body only.
        (
            "import { Lib } from './lib';\nnamespace H { const Lib = {}; export const p = Lib; }",
            vec![],
        ),
        (
            "import { Lib } from './lib';\nnamespace H { var Lib = {}; export const p = Lib; }",
            vec![],
        ),
        (
            "import { Lib } from './lib';\nnamespace H { const Lib = {}; }\nregister(Lib);",
            vec!["Lib"],
        ),
        (
            "import { Lib } from './lib';\nnamespace H { var Lib = {}; }\nregister(Lib);",
            vec!["Lib"],
        ),
        (
            "namespace Inner {}\nnamespace B { const Inner = 1; export const x = Inner; }",
            vec![],
        ),
        // A class that merges with a namespace of its own body hides nothing.
        (
            "namespace A { export class Inner {} export namespace Inner {} export const x = Inner; }",
            vec!["A.Inner"],
        ),
    ] {
        let facts = namespaces(source);
        assert_eq!(uses(&facts), used, "{source}");
    }
}

/// `typeof N`, `implements N.I` and `interface X extends N.I` name the
/// namespace in erased code, and a value use after them still counts.
#[test]
fn an_erased_type_name_is_no_use_of_a_namespace() {
    for source in [
        "namespace N {}\ntype T = typeof N;",
        "namespace N {}\ntype T = Array<typeof N.A>;",
        "namespace N { export interface I {} }\nclass C implements N.I {}",
        "namespace N { export interface I {} }\ninterface J extends N.I {}",
        "import { Lib } from './lib';\ntype T = typeof Lib;",
    ] {
        assert!(uses(&namespaces(source)).is_empty(), "{source}");
    }
    let facts = namespaces("namespace N {}\ntype T = typeof N;\nregister(N);");
    assert_eq!(uses(&facts), ["N"]);
}

/// `bind`, `call` and `apply` receive the member they are called on, so a
/// class they name leaves the graph's sight. A guard, or a receiver that is no
/// member, hands nothing on.
#[test]
fn a_member_handed_on_by_bind_call_or_apply_is_a_value_use() {
    for (source, used) in [
        ("namespace N {}\nnew (N.A.bind(null))();", vec!["N"]),
        ("namespace N {}\nN.A.call(null);", vec!["N"]),
        ("namespace N {}\nN.A.apply(null, []);", vec!["N"]),
        ("namespace N {}\nN['A'].bind(null);", vec!["N"]),
        ("namespace N {}\n(N.A as any).bind(null);", vec!["N"]),
        ("namespace N {}\nN.A.is(value);", vec![]),
        ("namespace N {}\nN.bind(null);", vec![]),
        ("import { fn } from './m';\nfn.bind(this);", vec![]),
    ] {
        let facts = namespaces(source);
        assert_eq!(uses(&facts), used, "{source}");
    }
}

/// A name read through one static member uses the export that member names, not
/// every export of the module, so the graph can escape just that one. A bare,
/// computed, wrapped or handed-on name still uses the whole of it, and a
/// declared namespace is escaped whole either way.
#[test]
fn a_name_read_through_a_static_member_is_a_use_of_that_member_alone() {
    for (source, read, whole) in [
        (
            "import * as mod from './m';\nconst v = mod.version;",
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
    ] {
        let facts = namespaces(source);
        assert_eq!(member_reads(&facts), read, "{source}");
        assert_eq!(uses(&facts), whole, "{source}");
    }
}

#[test]
fn constructions_in_a_namespace_body_are_sited_with_the_innermost_path() {
    let facts = namespaces(concat!(
        "namespace Outer.Inner {\n",
        "  export function make() { return new Local(); }\n",
        "  export class Local extends Error {}\n",
        "}\n",
        "export const outside = () => new Plain();\n",
    ));
    let sites: Vec<_> = facts
        .sites
        .iter()
        .map(|site| (site.namespace.as_str(), site.offset != 0))
        .collect();
    // Only constructions inside a namespace body are sited: the `new Local()`
    // and `Local`'s own synthetic `extends Error` construction (offset 0).
    // `new Plain()` sits outside every namespace and has no site.
    assert_eq!(sites, [("Outer.Inner", true), ("Outer.Inner", false)]);
    assert_eq!(facts.sites[1].caller_id, Some(facts.members[0].id));
}

#[test]
fn ambient_classes_are_the_ones_outside_a_tracked_namespace() {
    let allocator = Allocator::default();
    let source = concat!(
        "export namespace Tracked { export class A extends Error {} }\n",
        "declare class Declared extends Error {}\n",
        "declare module 'x' { class InModule extends Error {} }\n",
        "declare global { class InGlobal extends Error {} }\n",
        "declare namespace Ambient { class InAmbient extends Error {} }\n",
        "namespace Plain { function f() { class InFunction extends Error {} } }\n",
        "class Top extends Error {}\n",
    );
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let ambient: Vec<_> = facts
        .namespaces
        .unreported_class_ids
        .iter()
        .map(|class_id| {
            facts
                .callable_scope_ids
                .iter()
                .find_map(|(id, scope)| (id == class_id).then_some(scope.as_str()))
                .expect("class scope")
        })
        .collect();
    let mut ambient = ambient;
    ambient.sort_unstable();
    assert_eq!(
        ambient,
        [
            "Declared",
            "InAmbient",
            "InGlobal",
            "InModule",
            "f/InFunction"
        ]
    );
}

#[test]
fn a_required_module_is_recorded_as_used_whole() {
    let facts = namespaces(concat!(
        "import legacy = require('./legacy');\n",
        "export import other = require('./other');\n",
        "import alias = Ns.Inner;\n",
        "namespace Ns { export namespace Inner {} }\n",
    ));
    assert_eq!(facts.opaque_specifiers, ["./legacy", "./other"]);
}

#[test]
fn an_erased_type_only_import_equals_uses_no_module() {
    let facts = namespaces("import type erased = require('./erased');\n");
    assert!(facts.opaque_specifiers.is_empty());
}

/// `export { X as Y } from "./m"` names `m`'s export, never a local `X`.
#[test]
fn a_sourced_export_clause_does_not_export_a_local_namespace() {
    let facts = namespaces(concat!(
        "namespace Hidden { export class A extends Error {} }\n",
        "export { Hidden as Public } from './m';\n",
    ));
    assert_eq!(members(&facts), [("Hidden.A", false)]);
    assert!(facts.roots[0].exports.is_empty());
}

#[test]
fn an_import_of_the_same_name_merges_with_the_namespace() {
    let facts = namespaces("import { Ns } from './m';\nnamespace Ns { export class A {} }\n");
    assert!(facts.roots[0].merged);
}

#[test]
fn a_file_without_namespaces_or_imports_has_no_namespace_facts() {
    let facts = namespaces("class A extends Error {}\nconst x = new A();\nexport { x };\n");
    assert_eq!(facts, NamespaceFacts::default());
}
