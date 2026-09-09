use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn instance_this_member_calls_keep_this_receiver_spelling() {
    let facts = facts("class Service { run() { this.load(); } load() {} }");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "this.load" && call.caller.as_deref() == Some("Service/run")));
}

#[test]
fn constructor_this_member_calls_keep_this_receiver_spelling() {
    let facts = facts("class Service { constructor() { this.load(); } load() {} }");
    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "this.load" && call.caller.as_deref() == Some("Service/constructor")
    }));
}

#[test]
fn mixed_instance_and_static_run_keep_distinct_this_member_caller_ids() {
    let facts = facts(
        "class Service { run() { this.load(); } static run() { this.load(); } load() {} static load() {} }",
    );
    let loads: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "this.load")
        .collect();
    assert_eq!(loads.len(), 2);
    assert_ne!(loads[0].caller_id, loads[1].caller_id);
}

#[test]
fn derived_override_dynamic_imports_stay_on_the_overriding_method() {
    let facts = facts(
        "class Base { async load() { await import('./this-member-base-loaded.mts'); } } class Derived extends Base { constructor() { super(); this.load(); } async load() { await import('./this-member-loaded.mts'); } }",
    );
    let base = facts
        .imports
        .iter()
        .find(|import| import.specifier.contains("base-loaded"))
        .expect("base import");
    let derived = facts
        .imports
        .iter()
        .find(|import| import.specifier.ends_with("this-member-loaded.mts"))
        .expect("derived import");
    assert_eq!(base.function_scope.as_deref(), Some("Base/load"));
    assert_eq!(derived.function_scope.as_deref(), Some("Derived/load"));
}
