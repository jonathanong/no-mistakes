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
fn a_file_without_namespaces_or_imports_has_no_namespace_facts() {
    let facts = namespaces("class A extends Error {}\nconst x = new A();\nexport { x };\n");
    assert_eq!(facts, NamespaceFacts::default());
}
