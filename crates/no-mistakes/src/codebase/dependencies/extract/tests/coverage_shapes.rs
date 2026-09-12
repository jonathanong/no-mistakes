use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

fn tsx_facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn extract_walks_default_wrappers_assignment_aliases_and_syntax_edges() {
    let _ = facts(
        r#"
        export default foo!;
        export default <number>bar;
        export default (() => 1);
        export default (function wrapped() { return 1; });
        const helper = () => 1;
        const { fn = helper, ['computed']: skipped, missing } = { fn: helper, other: helper };
        const [first = helper, , third] = [helper, helper, helper];
        const { nested: { inner = helper } = { inner: helper } } = { nested: { inner: helper } };
        (helper)();
        helper[name]();
        class Box {
          accessor typed: number = helper();
          accessor empty;
          @deco
          accessor decorated = 1;
        }
        function deco(_target: unknown, _context: unknown) {}
        "#,
    );
    let _ = tsx_facts("export function Icon() { return <this.div />; }");
}

#[test]
fn extract_walks_class_heritage_dynamic_callees_and_resource_scopes() {
    let facts = facts(
        r#"
        function helper() {}
        function deco(_target: unknown) {}
        require.resolve("./mod");
        require("./cjs");
        (0, helper)();
        obj[dyn].method();
        new (obj[dyn].Cls)();
        new (function Anon() {})();
        obj[dyn].tag`x`;
        (0)`unknown`;
        foo().bar();

        class Base<T> {}
        class Box<T> extends Base<T> {
          value: number = helper();
          static field = () => 1;
          static get g() { return 1; }
          static set s(_v: number) {}
          [helper]() {}
          method() {}
        }
        class A extends B {
          static m() { return 1; }
        }
        class B extends A {}
        A.m();
        Box.constructor();

        export default function () {
          return helper();
        }
        export class Resource {
          method() {}
          field = () => 1;
          [computed]() {}
        }
        export const resource = {
          nested: {
            fn() {},
            [computed]: () => 1,
          },
        };

        @(factory())
        class FactoryDecorated {}
        "#,
    );
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "A.m" || call.callee.ends_with(".m")));
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee.contains("constructor")));
    assert!(!facts.unknown_calls.is_empty());
    let jsx = tsx_facts("export function Icon() { return <ns:tag />; }");
    assert!(jsx.callable_scopes.iter().any(|scope| scope == "Icon"));
}

#[test]
fn extract_walks_shadowed_require_spreads_accessors_and_callbacks() {
    let empty_require = facts("require(); require.resolve();");
    assert!(empty_require.imports.is_empty());
    let extracted = facts(
        r#"
        function require(id: string) { return id; }
        require("./shadowed");
        require.resolve("./still-global");
        function helper() {}
        helper(...args);
        new helper(...args);
        schedule(helper);
        class Box {
          static get g() { return 1; }
          static set s(_v: number) {}
          static m() { return 1; }
        }
        const value = Box.g;
        Box.s = 1;
        Box.m();
        const Alias = Box;
        Alias.m();
        const nested = Box.m;
        nested();
        function outer() {
          function inner() {}
          inner();
        }
        ({ name: renamed, nested: { inner = helper }, ...rest } = source);
        [first = helper, , ...tail] = items;
        obj.prop = helper;
        obj[dyn] = helper;
        "#,
    );
    assert!(extracted
        .function_calls
        .iter()
        .any(|call| call.callee == "helper" && call.is_callback));
    assert!(extracted
        .function_calls
        .iter()
        .any(|call| call.callee == "Box.m" || call.callee.ends_with(".m")));
}

#[test]
fn extract_walks_type_only_imports_exports_defaults_and_process_cwd() {
    let extracted = facts(
        r#"
        import type { Foo } from "./types.mts";
        import { type Bar, Baz } from "./mixed.mts";
        export type { Foo } from "./types.mts";
        export { type Bar } from "./mixed.mts";
        import { spawn, exec, execFile, fork } from "child_process";
        spawn("node", [], { cwd: "/tmp" });
        exec("node", { cwd: "/tmp" });
        execFile("node", [], { cwd: "/tmp" });
        fork("worker.mts", [], { cwd: "/tmp" });
        const helper = () => 1;
        export default helper;
        export default function named() { helper(); }
        export default class Named {
          static m() { return 1; }
        }
        export default class { method() { helper(); } }
        export enum Kind { A, B }
        const obj = {
          get g() { return 1; },
          set s(_v: number) {},
          method() { helper(); },
          ...helper,
          [computed]: () => 1,
        };
        function outer() {
          class Box { static m() {} }
          function inner() { Box.m(); Box.constructor(); }
          inner();
        }
        class A extends B {}
        class B extends A {}
        A.missing();
        const Alias = Box.m;
        Named.m();
        obj.method();
        obj["g"];
        import {} from "./empty.mts";
        tagged`plain`;
        new (0);
        "#,
    );
    assert!(extracted
        .imports
        .iter()
        .any(|import| import.kind == ImportKind::Type));
    assert!(extracted
        .function_calls
        .iter()
        .any(|call| call.callee.contains("spawn")
            || call.callee.contains("exec")
            || call.callee.contains("fork")));
    let jsx = tsx_facts(
        "export function Icon(props: { n: typeof this }) { return <this.span><ns:item /></this.span>; }",
    );
    assert!(jsx.callable_scopes.iter().any(|scope| scope == "Icon"));
}

#[test]
fn extract_walks_local_scope_chains_anonymous_classes_and_default_kinds() {
    let extracted = facts(
        r#"
        function helper() {}
        helper();
        function outer() {
          function helper() {}
          function inner() { helper(); }
          inner();
        }
        function Base() {}
        class Child extends Base { static m() {} }
        Child.m();
        const Box = 1;
        Box.m();
        class Service { static m() {} constructor() { helper(); } get g() { return 1; } set s(_v: number) {} field = 1; }
        new Service();
        const Alias = Service;
        Alias.m();
        const nested = Service.m;
        nested();
        function run() { let fn = helper; fn = other; fn(); }
        export default helper;
        export default () => helper();
        export default function () { helper(); }
        export default class { method() { helper(); } }
        const obj = {
          get [computed]() { return 1; },
          set [computed](_v: number) {},
          [computed]: () => 1,
        };
        @(0)
        class DynDeco {}
        "#,
    );
    assert!(extracted
        .function_calls
        .iter()
        .any(|call| call.callee == "helper" && call.caller.is_none()));
    assert!(extracted
        .function_calls
        .iter()
        .any(|call| call.callee == "helper" && call.caller.as_deref() == Some("outer/inner")));
    assert!(extracted.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run") && call.target_identity == CallTargetIdentity::Unknown
    }));
}
