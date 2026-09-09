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
fn lexical_scope_parents_preserve_nested_alias_resolution_order() {
    let facts = facts(
        "function target() {} function outer() { const outerAlias = target; function inner() { const innerAlias = outerAlias; innerAlias(); } inner(); }",
    );
    let inner_alias = facts
        .callable_aliases
        .iter()
        .find(|alias| alias.local == "innerAlias")
        .expect("inner alias");
    let outer_alias = facts
        .callable_aliases
        .iter()
        .find(|alias| alias.local == "outerAlias")
        .expect("outer alias");
    let parents = facts
        .lexical_scope_parents
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();

    assert_eq!(
        parents.get(&inner_alias.binding_scope),
        Some(&Some(outer_alias.binding_scope)),
        "the inner binding must search its enclosing lexical frame before module scope"
    );
}

#[test]
fn reassignment_invalidates_calls_without_erasing_callable_identities() {
    let facts = facts(
        "function target() {} const saved = target; target = injected; target(); saved(); class Registry { reload() {} } Registry.reload = injected; Registry.reload();",
    );

    assert!(facts.callable_scopes.iter().any(|scope| scope == "target"));
    assert!(facts
        .callable_scopes
        .iter()
        .any(|scope| scope == "Registry/reload"));
    assert!(facts.function_calls.iter().all(|call| {
        !matches!(call.callee.as_str(), "target" | "Registry.reload")
            || call.target_identity != CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts
        .callable_aliases
        .iter()
        .any(|alias| { alias.local == "saved" && alias.target == "target" }));
}

#[test]
fn member_callee_unwraps_typescript_receiver_wrappers() {
    let facts = facts("import * as api from './api.mts'; (api as typeof api).run();");
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "api.run" && call.target_identity == CallTargetIdentity::ModuleExport
    }));
}

#[test]
fn dynamic_member_receiver_records_unknown_call_evidence() {
    let source = "function run() { factory().invoke(); new (factory().Handler)(); }";
    let facts = facts(source);

    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "factory" && call.caller.as_deref() == Some("run")));
    assert_eq!(facts.unknown_calls.len(), 2);
    assert_eq!(facts.unknown_calls[0].caller.as_deref(), Some("run"));
    assert_eq!(facts.unknown_calls[0].line, 1);
    assert_eq!(facts.unknown_calls[0].invocation, InvocationKind::Call);
    assert_eq!(facts.unknown_calls[1].invocation, InvocationKind::Construct);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "<unknown>.invoke"));
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

#[test]
fn callable_var_uses_its_hoisted_function_binding_identity() {
    let facts = facts("function run() { { var load = () => {}; } load(); }");
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("run") && call.callee == "load")
        .expect("hoisted var call");

    assert_eq!(call.target_identity, CallTargetIdentity::RepositoryFunction);
}

#[test]
fn function_body_bindings_do_not_shadow_parameter_default_calls() {
    let facts = facts(
        "import { target } from './dep.mts'; function run(value = target()) { const target = local; }",
    );
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("run") && call.callee == "target")
        .expect("parameter default call");

    assert_eq!(call.target_identity, CallTargetIdentity::ModuleExport);
}

#[test]
fn local_class_members_are_callable_through_the_class_binding() {
    let facts = facts("class Service { static run() {} } Service.run();");
    assert_eq!(facts.class_scopes, ["Service"]);
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Service.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn static_getter_reads_are_recorded_separately_from_return_value_calls() {
    let facts = facts(
        "class Service { static get value() { return () => {}; } } function run() { Service.value(); }",
    );
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Service.value"
            && call.invocation == InvocationKind::Call
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.unknown_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run") && call.invocation == InvocationKind::Call
    }));
    assert_eq!(
        facts
            .function_calls
            .iter()
            .filter(|call| {
                call.caller.as_deref() == Some("run") && call.callee == "Service.value"
            })
            .count(),
        1
    );
}

#[test]
fn object_getter_reads_are_recorded_separately_from_return_value_calls() {
    let facts = facts(
        "const registry = { get value() { return () => {}; } }; function run() { registry.value(); }",
    );
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "registry.value"
            && call.invocation == InvocationKind::Call
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.unknown_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run") && call.invocation == InvocationKind::Call
    }));
    assert_eq!(
        facts
            .function_calls
            .iter()
            .filter(|call| {
                call.caller.as_deref() == Some("run") && call.callee == "registry.value"
            })
            .count(),
        1
    );
}

#[test]
fn reassigned_object_binding_does_not_invoke_its_previous_getter() {
    let facts = facts(
        "let registry = { get value() { return () => {}; } }; registry = {}; function run() { return registry.value; }",
    );

    assert!(!facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "registry.value"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn static_setter_assignment_invokes_the_setter_without_invalidating_it() {
    let facts = facts(
        "class Service { static set value(next) {} } function run() { Service.value = 1; Service.value = 2; }",
    );
    let setter_calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "Service.value")
        .collect();

    assert_eq!(setter_calls.len(), 2);
    assert!(setter_calls.iter().all(|call| {
        call.invocation == InvocationKind::Call
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn computed_class_keys_retain_class_symbol_ownership() {
    let facts =
        facts("import { alpha as key } from './source.mts'; export class Client { [key]() {} }");
    let class_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "Client").then_some(*id))
        .expect("class callable identity");
    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("Client")
            && reference.caller_id == Some(class_id)
            && reference.callee == "key"
    }));
}

#[test]
fn only_static_class_members_are_callable_through_the_class_binding() {
    let facts =
        facts("class Service { run() {} static reload() {} } Service.run(); Service.reload();");

    assert!(!facts.function_calls.iter().any(|call| {
        call.callee == "Service.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Service.reload"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn only_static_class_expression_members_are_callable_through_the_binding() {
    let facts = facts(
        "const Service = class { run() {} static reload() {} }; Service.run(); Service.reload();",
    );

    assert!(!facts.function_calls.iter().any(|call| {
        call.callee == "Service.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "Service.reload"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn nested_local_class_members_are_callable_through_the_class_binding() {
    let facts = facts("function boot() { class Service { static run() {} } Service.run(); }");
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("boot")
            && call.callee == "Service.run"
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts
        .class_scopes
        .iter()
        .any(|scope| scope == "boot/Service"));
}

#[test]
fn sibling_block_callables_share_display_names_but_have_distinct_ids() {
    let facts = facts(
        "function outer() { { function f() { first(); } f(); } { function f() { second(); } f(); } }",
    );
    let scopes: Vec<_> = facts
        .callable_scopes
        .iter()
        .filter(|scope| scope.ends_with("/f"))
        .collect();

    assert_eq!(scopes, ["outer/f"]);
    let ids: Vec<_> = facts
        .callable_scope_ids
        .iter()
        .filter_map(|(id, scope)| (scope == "outer/f").then_some(*id))
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
    let first_caller = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("outer/f") && call.callee == "first")
        .expect("call in first sibling");
    let second_caller = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("outer/f") && call.callee == "second")
        .expect("call in second sibling");
    assert_ne!(first_caller.caller_id, second_caller.caller_id);
    assert!(ids.contains(&first_caller.caller_id.expect("first sibling id")));
    assert!(ids.contains(&second_caller.caller_id.expect("second sibling id")));
}
