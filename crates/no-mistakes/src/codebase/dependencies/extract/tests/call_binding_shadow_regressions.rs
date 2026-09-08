use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn lexical_binding_identity_stops_outer_callable_lookup() {
    let facts = facts("function target() {} function run(target) { target(); }");
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("run") && call.callee == "target")
        .expect("parameter call");

    assert_eq!(call.target_identity, CallTargetIdentity::Unknown);
    assert!(call.callee_binding_scope.is_some());
}

#[test]
fn nested_block_shadow_has_a_distinct_callee_binding_identity() {
    let facts = facts(
        "function target() {} function run() { const alias = target; alias(); { const alias = value; alias(); } }",
    );
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "alias")
        .collect();

    assert_eq!(calls.len(), 2);
    assert_ne!(calls[0].callee_binding_scope, calls[1].callee_binding_scope);
    assert_eq!(calls[1].target_identity, CallTargetIdentity::Unknown);
    assert_eq!(facts.callable_aliases.len(), 1);
    assert_eq!(
        facts.callable_aliases[0].binding_scope,
        calls[0].callee_binding_scope.unwrap()
    );
}

#[test]
fn reassignment_invalidates_callable_declarations_and_members() {
    let facts = facts(
        "function target() {} target = injected; target(); class Registry { reload() {} } Registry.reload = injected; Registry.reload();",
    );

    assert!(!facts.callable_scopes.iter().any(|scope| scope == "target"));
    assert!(!facts
        .callable_scopes
        .iter()
        .any(|scope| scope == "Registry/reload"));
    assert!(facts.function_calls.iter().all(|call| {
        !matches!(call.callee.as_str(), "target" | "Registry.reload")
            || call.target_identity != CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn member_callee_unwraps_typescript_receiver_wrappers() {
    let facts = facts("import * as api from './api.mts'; (api as typeof api).run();");
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "api.run" && call.target_identity == CallTargetIdentity::ModuleExport
    }));
}

#[test]
fn aggregate_membership_is_not_an_invocation_callback() {
    let facts = facts("const registry = { load() {} };");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.invocation == InvocationKind::Membership));
}

#[test]
fn shadowed_require_is_recorded_once() {
    let facts = facts("function run(require) { require('x'); }");
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "require")
        .collect();

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].target_identity, CallTargetIdentity::Unknown);
}

#[test]
fn named_function_expression_self_reference_aliases_its_variable_scope() {
    let facts = facts("const factorial = function recur(n) { return recur(n - 1); };");
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "recur")
        .expect("recursive self call");

    assert_eq!(call.caller.as_deref(), Some("factorial"));
    assert_eq!(call.target_identity, CallTargetIdentity::RepositoryFunction);
    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.scope.as_deref() == Some("factorial")
            && alias.local == "recur"
            && alias.target == "factorial"
            && alias.binding_scope == call.callee_binding_scope.expect("self binding scope")
    }));
}

#[test]
fn eager_object_initializers_are_module_owned_but_function_properties_are_scoped() {
    let facts =
        facts("function target() {} const config = { value: target(), load() { target(); } };");
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "target")
        .collect();

    assert_eq!(calls.len(), 2);
    assert!(calls.iter().any(|call| call.caller.is_none()));
    assert!(calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("config/load")));
}
