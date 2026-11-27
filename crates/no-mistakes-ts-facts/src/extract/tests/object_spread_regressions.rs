use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn reassigned_object_spread_stays_conservative() {
    let facts =
        facts("let source = { run() {} }; source = {}; const api = { ...source }; api.run();");
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "api.run"));
}

#[test]
fn class_object_spread_stays_conservative() {
    let facts = facts("class Source { static run() {} } const api = { ...Source }; api.run();");
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "api.run"));
}

#[test]
fn unknown_object_literal_spread_call_stays_unresolved() {
    let facts = facts("function factory() { return { run() {} }; } ({ ...factory() }).run();");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "<unknown>.run"));
}

#[test]
fn object_literal_named_property_call_resolves_through_the_value() {
    let facts = facts("function target() {} ({ run: target }).run();");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "target"));
}

#[test]
fn object_literal_spread_method_call_resolves_to_the_source_member() {
    let facts = facts("const source = { run() {} }; ({ ...source }).run();");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "source.run"));
}

#[test]
fn object_spread_copies_accessor_members() {
    let facts = facts(
        "const source = { get value() {}, set value(next) {} }; const api = { ...source }; api.value; api.value = 1;",
    );
    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "api.value" && alias.target == "source.value"));
}

#[test]
fn later_unknown_spread_clears_copied_object_members() {
    let facts = facts(
        "const source = { run() {} }; function factory() { return {}; } const api = { ...source, ...factory() }; api.run();",
    );
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "api.run"));
}

#[test]
fn later_explicit_property_wins_after_unknown_object_literal_spread() {
    let facts = facts(
        "function target() {} function factory() { return {}; } ({ ...factory(), run: target }).run();",
    );
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "target"));
}

#[test]
fn nested_source_shadow_copies_inner_spread_aliases_only() {
    let facts = facts(
        "function outerTarget() {} function innerTarget() {} const source = { run: outerTarget }; function wrap() { const source = { run: innerTarget }; const api = { ...source }; api.run(); }",
    );
    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "api.run"
            && alias.target == "source.run"
            && alias.scope.as_deref() == Some("wrap")
    }));
    assert!(!facts
        .callable_aliases
        .iter()
        .any(|alias| { alias.local == "api.run" && alias.target == "outerTarget" }));
}
