use super::*;

fn facts(source: &str) -> ImportFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    extract_import_facts_from_program_with_source(&parsed.program, source)
}

#[test]
fn inline_export_function_predeclaration_records_its_callable_binding_id() {
    let facts = facts("helper(); export function helper() {}");
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.is_none() && call.callee == "helper")
        .expect("same-file helper call");
    let binding_scope = call.callee_binding_scope.expect("helper binding scope");
    let helper_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "helper").then_some(*id))
        .expect("helper callable identity");

    assert_eq!(call.target_identity, CallTargetIdentity::RepositoryFunction);
    assert!(facts.callable_bindings.iter().any(|(scope, name, id)| {
        *scope == binding_scope && name == "helper" && *id == helper_id
    }));
}

#[test]
fn manually_walked_callable_bodies_predeclare_later_function_and_shadow_bindings() {
    let facts = facts(
        "const arrow = () => { helper(); function helper() {} setTimeout(); const setTimeout = local; };\
         const expression = function() { helper(); function helper() {} setTimeout(); let setTimeout = local; };\
         const aggregate = { arrow: () => { helper(); function helper() {} setTimeout(); const setTimeout = local; } };",
    );

    assert_eq!(
        facts
            .function_calls
            .iter()
            .filter(|call| call.callee == "helper")
            .count(),
        3
    );
    assert!(facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "helper")
        .all(|call| call.target_identity == CallTargetIdentity::RepositoryFunction));
    assert!(facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "setTimeout")
        .all(|call| call.target_identity == CallTargetIdentity::Unknown));
}

#[test]
fn class_method_bodies_predeclare_later_function_and_shadow_bindings() {
    let facts = facts(
        "export default (class {\
           static run() {\
             helper();\
             function helper() {}\
             setTimeout();\
             const setTimeout = local;\
           }\
         });",
    );

    let helper_call = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "helper")
        .expect("class method helper call");
    assert_eq!(helper_call.caller.as_deref(), Some("default/run"));
    assert_eq!(
        helper_call.target_identity,
        CallTargetIdentity::RepositoryFunction
    );

    let timeout_call = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "setTimeout")
        .expect("class method shadowed call");
    assert_eq!(timeout_call.caller.as_deref(), Some("default/run"));
    assert_eq!(timeout_call.target_identity, CallTargetIdentity::Unknown);
}
