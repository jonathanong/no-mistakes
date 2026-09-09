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
