use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "parse errors: {:#?}",
        parsed.diagnostics
    );
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn nested_class_eager_expressions_keep_the_enclosing_callable_owner() {
    let facts = facts(
        "function outer() { class Service extends loadBase() { static value = loadStatic(); static { loadBlock(); } [loadKey()]() {} method() { loadMethod(); } } }",
    );

    for callee in ["loadBase", "loadStatic", "loadBlock", "loadKey"] {
        assert!(facts
            .function_calls
            .iter()
            .any(|call| { call.callee == callee && call.caller.as_deref() == Some("outer") }));
    }
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "loadMethod" && call.caller.as_deref() == Some("outer/Service/method")
    }));
}

#[test]
fn inherited_static_callable_members_resolve_through_local_base_classes() {
    let facts = facts("class Base { static run() {} } class Child extends Base {} Child.run();");

    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "Child.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction));
}

#[test]
fn named_class_and_member_decorators_are_calls_in_the_enclosing_evaluation_scope() {
    let facts = facts(
        r#"
            import { classDecorator } from "./class-decorator.mts";
            import * as memberDecorators from "./member-decorator.mts";

            function localDecorator() {}
            function outer() {
              @classDecorator
              class Service {
                @memberDecorators.decorate
                method() {}

                @localDecorator
                field = 1;
              }
            }
        "#,
    );

    for (callee, identity) in [
        ("classDecorator", CallTargetIdentity::ModuleExport),
        (
            "memberDecorators.decorate",
            CallTargetIdentity::ModuleExport,
        ),
        ("localDecorator", CallTargetIdentity::RepositoryFunction),
    ] {
        assert!(
            facts.function_calls.iter().any(|call| {
                call.callee == callee
                    && call.caller.as_deref() == Some("outer")
                    && call.invocation == InvocationKind::Call
                    && call.target_identity == identity
            }),
            "{callee} must be an enclosing-scope decorator invocation: {:#?}",
            facts.function_calls
        );
    }
}

#[test]
fn dynamic_decorators_remain_unknown_calls_without_named_edges() {
    let facts = facts(
        r#"
            function outer() {
              @factory()
              class FactoryDecorated {}

              @(decorators[method])
              class ComputedDecorated {}
            }
        "#,
    );

    assert!(
        facts
            .function_calls
            .iter()
            .any(|call| { call.callee == "factory" && call.caller.as_deref() == Some("outer") }),
        "decorator factory call must retain its enclosing owner: {:#?}",
        facts.function_calls
    );
    assert_eq!(
        facts
            .unknown_calls
            .iter()
            .filter(|call| call.caller.as_deref() == Some("outer"))
            .count(),
        2,
        "factory-result and computed decorators must stay unresolved"
    );
}

#[test]
fn overload_signatures_share_the_implementation_callable_id() {
    let facts = facts(
        "export function parse(value: string): string; export function parse(value: number): string; export function parse(value: string | number) { return target(); } function target() {} parse('input');",
    );
    let parse_ids: Vec<_> = facts
        .callable_scope_ids
        .iter()
        .filter_map(|(id, scope)| (scope == "parse").then_some(*id))
        .collect();

    assert_eq!(parse_ids.len(), 1);
    assert!(facts.function_calls.iter().any(|call| {
        call.caller_id == Some(parse_ids[0])
            && call.callee == "target"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn class_method_overload_signatures_share_the_implementation_callable_id() {
    let facts = facts(
        "class Parser { parse(value: string): string; parse(value: number): string; parse(value: string | number) { return target(); } } function target() {} new Parser().parse('input');",
    );
    let parse_ids: Vec<_> = facts
        .callable_scope_ids
        .iter()
        .filter_map(|(id, scope)| (scope == "Parser/parse").then_some(*id))
        .collect();

    assert_eq!(parse_ids.len(), 1);
    assert!(facts.function_calls.iter().any(|call| {
        call.caller_id == Some(parse_ids[0])
            && call.callee == "target"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn bound_anonymous_class_overloads_share_implementation_callable_ids() {
    let facts = facts(
        "const Parser: import('./parser-types.mts').Parser = class { constructor(value: string); constructor(value: number); constructor(value: string | number) {} parse(value: string): string; parse(value: number): string; parse(value: string | number) { return target(); } }; function target() {} new Parser('input').parse('input');",
    );

    for scope in ["Parser/constructor", "Parser/parse"] {
        assert_eq!(
            facts
                .callable_scope_ids
                .iter()
                .filter(|(_, candidate)| candidate == scope)
                .count(),
            1,
            "{scope} must use only its implementation identity: {:?}",
            facts.callable_scope_ids
        );
    }
    assert!(facts
        .imports
        .iter()
        .any(|import| import.specifier == "./parser-types.mts" && import.kind == ImportKind::Type));
}

#[test]
fn bound_named_class_expression_keeps_its_internal_identity() {
    let facts = facts(
        "const Public = class Internal { constructor() {} static load() {} parse(value: string): string; parse(value: number): string; parse(value: string | number) { Internal.load(); return ''; } }; new Public().parse('input'); Public.load(); function outer() { const NestedPublic = class NestedInternal { constructor() {} static load() {} parse(value: string): string; parse(value: number): string; parse(value: string | number) { NestedInternal.load(); return ''; } }; new NestedPublic().parse('input'); NestedPublic.load(); }",
    );

    for (outward, internal, scope) in [
        ("Public", "Internal", "Internal"),
        ("NestedPublic", "NestedInternal", "outer/NestedInternal"),
    ] {
        let outward_id = facts
            .callable_bindings
            .iter()
            .find_map(|(_, name, id)| (name == outward).then_some(*id))
            .expect("outward class binding");
        let internal_id = facts
            .callable_bindings
            .iter()
            .find_map(|(_, name, id)| (name == internal).then_some(*id))
            .expect("internal class binding");
        assert_eq!(outward_id, internal_id, "{outward} and {internal}");

        let method_scope = format!("{scope}/parse");
        assert_eq!(
            facts
                .callable_scope_ids
                .iter()
                .filter(|(_, candidate)| candidate == &method_scope)
                .count(),
            1,
            "{method_scope} must collapse overloads to its implementation ID"
        );
        assert!(!facts
            .callable_scope_ids
            .iter()
            .any(|(_, candidate)| candidate == &format!("{outward}/parse")));
        for callee in [outward.to_string(), format!("{outward}.load")] {
            assert!(facts.function_calls.iter().any(|call| {
                call.callee == callee
                    && call.target_identity == CallTargetIdentity::RepositoryFunction
            }));
        }
        assert!(facts.function_calls.iter().any(|call| {
            call.caller.as_deref() == Some(method_scope.as_str())
                && call.callee == format!("{internal}.load")
                && call.target_identity == CallTargetIdentity::RepositoryFunction
        }));
    }
}

#[test]
fn nested_aggregate_callables_keep_their_owner_and_member_identity() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase/dependencies/nested-callable-aggregates/fixture/src/aggregate-callables.mts");
    let source = std::fs::read_to_string(&fixture).expect("aggregate callable fixture must exist");
    let facts = facts(&source);

    for scope in [
        "boot/registry/load",
        "boot/Service/run",
        "boot/Service/reload",
    ] {
        assert!(
            facts
                .callable_scope_ids
                .iter()
                .any(|(_, candidate)| candidate == scope),
            "{scope} must retain a callable identity: {:?}",
            facts.callable_scope_ids
        );
    }

    let registry_id = facts
        .callable_bindings
        .iter()
        .find_map(|(_, binding, id)| (binding == "registry").then_some(*id))
        .expect("nested registry binding");
    assert!(facts.function_calls.iter().any(|call| {
        call.invocation == InvocationKind::Membership
            && call.caller.as_deref() == Some("boot/registry")
            && call.caller_id == Some(registry_id)
            && call.callee == "load"
    }));
    for callee in ["registry.load", "Service.run", "Service.reload"] {
        assert!(
            facts.function_calls.iter().any(|call| {
                call.callee == callee
                    && call.target_identity == CallTargetIdentity::RepositoryFunction
            }),
            "{callee} must resolve to its aggregate callable: {:#?}",
            facts.function_calls
        );
    }
    assert!(facts.imports.iter().any(|import| {
        import.specifier == "./object-called.mts"
            && import.function_scope.as_deref() == Some("boot/registry/load")
    }));

    let class_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "boot/Service").then_some(*id))
        .expect("nested class identity");
    for (member, specifier) in [
        ("run", "./field-called.mts"),
        ("reload", "./field-reloaded.mts"),
    ] {
        let member_id = facts
            .imports
            .iter()
            .find_map(|import| {
                (import.specifier == specifier
                    && import.function_scope.as_deref() == Some(&format!("boot/Service/{member}")))
                .then_some(import.function_scope_id)
            })
            .flatten()
            .expect("static field callable owner");
        assert!(
            facts.class_member_callable_ids.iter().any(
                |(candidate_class, candidate_member, candidate_id)| {
                    *candidate_class == class_id
                        && candidate_member == member
                        && *candidate_id == member_id
                }
            ),
            "{member} must preserve its field callable identity"
        );
    }
}
