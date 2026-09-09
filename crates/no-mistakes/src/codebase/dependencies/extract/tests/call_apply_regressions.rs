use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

fn recorded_callees(source: &str) -> Vec<String> {
    facts(source)
        .function_calls
        .into_iter()
        .filter(|call| call.invocation == InvocationKind::Call)
        .map(|call| call.callee)
        .collect()
}

#[test]
fn local_function_call_normalizes_to_the_function() {
    let callees = recorded_callees("function target() {} target.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "target"));
    assert!(!callees.iter().any(|callee| callee == "target.call"));
}

#[test]
fn local_function_apply_normalizes_to_the_function() {
    let callees = recorded_callees("function target() {} target.apply(undefined, []);");
    assert!(callees.iter().any(|callee| callee == "target"));
    assert!(!callees.iter().any(|callee| callee == "target.apply"));
}

#[test]
fn aliased_function_call_normalizes_to_the_alias() {
    let callees =
        recorded_callees("function target() {} const alias = target; alias.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "alias"));
    assert!(!callees.iter().any(|callee| callee == "alias.call"));
}

#[test]
fn imported_function_call_normalizes_to_the_import() {
    let callees =
        recorded_callees("import { target } from './target.mts'; target.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "target"));
    assert!(!callees.iter().any(|callee| callee == "target.call"));
}

#[test]
fn imported_function_apply_normalizes_to_the_import() {
    let callees =
        recorded_callees("import { target } from './target.mts'; target.apply(undefined, []);");
    assert!(callees.iter().any(|callee| callee == "target"));
}

#[test]
fn predeclared_imported_function_call_normalizes_to_the_import() {
    let callees =
        recorded_callees("target.call(undefined); import { target } from './target.mts';");
    assert!(callees.iter().any(|callee| callee == "target"));
}

#[test]
fn function_prototype_call_stays_on_the_helper() {
    let callees = recorded_callees("Function.prototype.call(undefined);");
    assert!(callees
        .iter()
        .any(|callee| callee == "Function.prototype.call"));
}

#[test]
fn computed_call_property_normalizes_like_static_call() {
    let callees = recorded_callees("function target() {} target['call'](undefined);");
    assert!(callees.iter().any(|callee| callee == "target"));
}

#[test]
fn nested_call_helpers_normalize_to_the_function() {
    let callees = recorded_callees("function target() {} target.call.call(undefined, undefined);");
    assert!(callees.iter().any(|callee| callee == "target"));
}

#[test]
fn object_own_call_member_is_not_rewritten() {
    let callees = recorded_callees("const api = { call() {} }; api.call();");
    assert!(callees.iter().any(|callee| callee == "api.call"));
}

#[test]
fn object_own_apply_member_is_not_rewritten() {
    let callees = recorded_callees("const api = { apply() {} }; api.apply();");
    assert!(callees.iter().any(|callee| callee == "api.apply"));
}

#[test]
fn class_call_helper_is_not_rewritten() {
    let callees = recorded_callees("class Service {} Service.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "Service.call"));
}

#[test]
fn unknown_receiver_call_stays_unresolved() {
    let callees = recorded_callees("function factory() { return () => {}; } factory().call();");
    assert!(callees.iter().any(|callee| callee == "<unknown>.call"));
}

#[test]
fn this_call_is_not_rewritten() {
    let callees = recorded_callees("class Service { run() { this.call(); } }");
    assert!(callees.iter().any(|callee| callee == "this.call"));
}

#[test]
fn bind_is_not_treated_as_a_call_helper() {
    let callees = recorded_callees("function target() {} target.bind(undefined);");
    assert!(callees.iter().any(|callee| callee == "target.bind"));
}

#[test]
fn object_member_call_helper_normalizes_to_the_member() {
    let callees = recorded_callees("const api = { run() {} }; api.run.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "api.run"));
    assert!(!callees.iter().any(|callee| callee == "api.run.call"));
}

#[test]
fn aliased_object_call_member_is_not_rewritten() {
    let callees =
        recorded_callees("function target() {} const api = { call: target }; api.call();");
    assert!(callees.iter().any(|callee| callee == "api.call"));
}

#[test]
fn object_getter_call_member_is_not_rewritten() {
    let facts = facts("const api = { get call() { return () => {}; } }; api.call();");
    assert!(!facts
        .function_calls
        .iter()
        .any(|call| { call.invocation == InvocationKind::Call && call.callee == "api" }));
}

#[test]
fn object_setter_call_member_is_not_rewritten() {
    let callees = recorded_callees("const api = { set call(value) {} }; api.call();");
    assert!(callees.iter().any(|callee| callee == "api.call"));
}

#[test]
fn class_static_member_call_helper_normalizes_to_the_member() {
    let callees =
        recorded_callees("class Service { static run() {} } Service.run.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "Service.run"));
    assert!(!callees.iter().any(|callee| callee == "Service.run.call"));
}

#[test]
fn class_alias_static_member_call_helper_normalizes_to_the_member() {
    let callees = recorded_callees(
        "class Service { static run() {} } const Alias = Service; Alias.run.call(undefined);",
    );
    assert!(callees.iter().any(|callee| callee == "Alias.run"));
}

#[test]
fn non_callable_binding_call_helper_still_normalizes() {
    let callees = recorded_callees("const value = 1; value.call(undefined);");
    assert!(callees.iter().any(|callee| callee == "value"));
}
