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

#[test]
fn static_accessors_keep_kind_specific_collector_maps() {
    let collector = include_str!("../../extract_collector.rs");
    assert!(
        collector.contains(
            "static_getter_member_ids: FxHashMap<CallableId, FxHashMap<String, CallableId>>"
        ),
        "static getters must nest by owner so paired accessors keep distinct identities"
    );
    assert!(
        collector.contains(
            "static_setter_member_ids: FxHashMap<CallableId, FxHashMap<String, CallableId>>"
        ),
        "static setters must nest by owner so paired accessors keep distinct identities"
    );
}

#[test]
fn computed_assignment_keys_still_read_static_getters() {
    let facts = facts(
        "class Registry { static get value() {} static set value(next) {} } const target = {}; function run() { target[Registry.value] = 1; }",
    );
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Registry.value"
            && call.invocation == InvocationKind::Get
    }));
    assert!(!facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Registry.value"
            && call.invocation == InvocationKind::Set
    }));
}

#[test]
fn string_computed_static_getter_reads_are_recorded() {
    let facts = facts(
        "class Registry { static get current() { import('./dep.mts'); } } function run() { return Registry['current']; }",
    );
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Registry.current"
            && call.invocation == InvocationKind::Get
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn dynamic_computed_static_getter_reads_stay_unresolved() {
    let facts = facts(
        "class Registry { static get current() { import('./dep.mts'); } } const name = 'current'; function run() { return Registry[name]; }",
    );
    assert!(!facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Registry.current"
            && call.invocation == InvocationKind::Get
    }));
}

#[test]
fn object_setter_assignment_invokes_the_setter_without_invalidating_it() {
    let facts = facts(
        "const api = { set value(next) { import('./dep.mts'); } }; function run() { api.value = 1; api['value'] = 2; }",
    );
    let setter_calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "api.value")
        .collect();

    assert_eq!(setter_calls.len(), 2);
    assert!(setter_calls.iter().all(|call| {
        call.invocation == InvocationKind::Set
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
    assert!(facts.imports.iter().any(|import| {
        import.specifier == "./dep.mts" && import.function_scope.as_deref() == Some("api/value")
    }));
}

#[test]
fn object_setter_update_invokes_the_setter_without_invalidating_it() {
    let facts = facts(
        "const api = { set value(next) { import('./dep.mts'); } }; function run() { api.value++; --api.value; }",
    );
    let setter_calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "api.value")
        .collect();

    assert_eq!(setter_calls.len(), 2);
    assert!(setter_calls.iter().all(|call| {
        call.invocation == InvocationKind::Set
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn object_data_property_writes_invalidate_the_replaced_member() {
    let facts =
        facts("const api = { load() {} }; function run() { api.load = () => {}; api.load(); }");
    assert!(!facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "api.load"
            && call.invocation == InvocationKind::Set
    }));
    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "api.load"
            && call.target_identity == CallTargetIdentity::Unknown
    }));
}

#[test]
fn getter_calls_and_cross_kind_accessors_stay_unresolved_or_unknown() {
    let getter_call = facts(
        "class Service { static get value() { return () => 1; } } const api = { get g() { return 1; } }; function run() { Service.value(); api.g(); }",
    );
    assert!(getter_call.unknown_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run") && call.invocation == InvocationKind::Call
    }));

    let setter_only =
        facts("class Service { static set value(next) {} } function run() { Service.value; }");
    assert!(!setter_only.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Service.value"
            && call.invocation == InvocationKind::Get
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));

    let getter_only_write = facts(
        "class Service { static get value() { return 1; } } function run() { Service.value = 1; }",
    );
    assert!(!getter_only_write.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "Service.value"
            && call.invocation == InvocationKind::Set
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}
