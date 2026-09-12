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
