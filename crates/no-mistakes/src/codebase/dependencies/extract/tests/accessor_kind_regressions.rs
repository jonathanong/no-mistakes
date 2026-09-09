use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn paired_static_accessors_keep_distinct_callable_ids() {
    let facts = facts(
        "class Service { static get value() { import('./get.mts'); } static set value(next) { import('./set.mts'); } } function run() { Service.value; Service.value = 1; }",
    );
    let class_id = facts
        .callable_bindings
        .iter()
        .find_map(|(_, binding, id)| (binding == "Service").then_some(*id))
        .expect("Service class identity");
    let getter_id = facts
        .static_getter_callable_ids
        .iter()
        .find_map(|(owner, member, id)| (*owner == class_id && member == "value").then_some(*id))
        .expect("getter identity");
    let setter_id = facts
        .static_setter_callable_ids
        .iter()
        .find_map(|(owner, member, id)| (*owner == class_id && member == "value").then_some(*id))
        .expect("setter identity");

    assert_ne!(getter_id, setter_id);
    assert!(!facts
        .class_member_callable_ids
        .iter()
        .any(|(owner, member, _)| *owner == class_id && member == "value"));
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Service.value"
            && call.invocation == InvocationKind::Get
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Service.value"
            && call.invocation == InvocationKind::Set
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}
