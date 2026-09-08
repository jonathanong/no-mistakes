use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
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
        "const Public = class Internal { static load() {} parse(value: string): string; parse(value: number): string; parse(value: string | number) { Internal.load(); return ''; } }; new Public().parse('input');",
    );

    assert_eq!(
        facts
            .callable_scope_ids
            .iter()
            .filter(|(_, scope)| scope == "Internal/parse")
            .count(),
        1
    );
    assert!(!facts
        .callable_scope_ids
        .iter()
        .any(|(_, scope)| scope == "Public/parse"));
    assert!(
        facts.function_calls.iter().any(|call| {
            call.caller.as_deref() == Some("Internal/parse") && call.callee == "Internal.load"
        }),
        "named class self-reference must remain in the internal method scope: {:#?}",
        facts.function_calls
    );
}
