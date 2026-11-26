use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn reassignment_of_an_inner_shadow_does_not_invalidate_the_outer_callable() {
    let facts = facts(
        "function target() {} function run() { { let target = injected; target = replaced; target(); } target(); }",
    );

    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "target")
        .collect();

    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].target_identity, CallTargetIdentity::Unknown);
    assert_eq!(
        calls[1].target_identity,
        CallTargetIdentity::RepositoryFunction
    );
    assert!(facts.callable_scopes.iter().any(|scope| scope == "target"));
}

#[test]
fn for_iteration_assignment_targets_invalidate_callable_bindings() {
    let facts = facts(
        "function target() {} function run() { for (target of values) { target(); } for (target in values) { target(); } }",
    );

    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "target")
        .collect();

    assert_eq!(calls.len(), 2);
    assert!(calls
        .iter()
        .all(|call| call.target_identity == CallTargetIdentity::Unknown));
}

#[test]
fn inner_alias_increment_does_not_invalidate_the_outer_callable() {
    let facts = facts(
        "function target() {} function run() { const alias = target; { const alias = value; alias++; } alias(); }",
    );
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.as_deref() == Some("run") && call.callee == "alias")
        .expect("outer alias call");

    assert_eq!(call.target_identity, CallTargetIdentity::RepositoryFunction);
    assert!(facts.callable_aliases.iter().any(|alias| {
        alias.local == "alias" && alias.target == "target" && alias.invalidated_at.is_none()
    }));
}

#[test]
fn var_bound_object_aggregate_binds_in_the_hoisted_function_scope() {
    let facts = facts(
        "function run() { { var api = { load() { import('./dep.mts'); }, unused() { import('./unused.mts'); } }; } api.load(); }",
    );
    let load = facts
        .function_calls
        .iter()
        .find(|call| {
            call.caller.as_deref() == Some("run")
                && call.callee == "api.load"
                && call.invocation == InvocationKind::Call
        })
        .expect("post-block aggregate call");
    let binding_scope = load.callee_binding_scope.expect("hoisted api binding");

    assert_eq!(load.target_identity, CallTargetIdentity::RepositoryFunction);
    assert!(facts
        .callable_bindings
        .iter()
        .any(|(scope, name, _)| *scope == binding_scope && name == "api"));
    assert!(facts.imports.iter().any(|import| {
        import.specifier == "./dep.mts" && import.function_scope.as_deref() == Some("run/api/load")
    }));
}
