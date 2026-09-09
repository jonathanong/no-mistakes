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
fn named_default_function_predeclaration_records_its_callable_binding_id() {
    let facts = facts("namedDefault(); export default function namedDefault() {}");
    let call = facts
        .function_calls
        .iter()
        .find(|call| call.caller.is_none() && call.callee == "namedDefault")
        .expect("same-file named default call");
    let binding_scope = call
        .callee_binding_scope
        .expect("named default binding scope");
    let default_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "namedDefault").then_some(*id))
        .expect("named default callable identity");

    assert_eq!(call.target_identity, CallTargetIdentity::RepositoryFunction);
    assert!(facts.callable_bindings.iter().any(|(scope, name, id)| {
        *scope == binding_scope && name == "namedDefault" && *id == default_id
    }));
}

#[test]
fn named_default_class_binds_in_the_module_scope() {
    let facts = facts(
        r#"export default class Service { constructor() { import("./dep.mts"); } unused() { import("./unused.mts"); } } new Service();"#,
    );
    let class_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "Service").then_some(*id))
        .expect("named default class identity");
    let construction = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "Service" && call.invocation == InvocationKind::Construct)
        .expect("module-scope construction");

    assert_eq!(
        construction.target_identity,
        CallTargetIdentity::RepositoryFunction
    );
    assert!(facts
        .callable_bindings
        .iter()
        .any(|(scope, name, id)| { *scope == 0 && name == "Service" && *id == class_id }));
    assert!(facts.imports.iter().any(|import| {
        import.specifier == "./dep.mts"
            && import.function_scope.as_deref() == Some("Service/constructor")
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

#[test]
fn wrapped_default_class_uses_its_class_scope_and_default_binding_identity() {
    let facts = facts(
        "export default ((class Wrapped { static run() { helper(); function helper() {} } }) satisfies unknown);",
    );
    let class_id = facts
        .callable_scope_ids
        .iter()
        .find_map(|(id, scope)| (scope == "Wrapped").then_some(*id))
        .expect("wrapped class callable identity");
    let helper_call = facts
        .function_calls
        .iter()
        .find(|call| call.callee == "helper")
        .expect("wrapped class method call");

    assert_eq!(helper_call.caller.as_deref(), Some("Wrapped/run"));
    assert!(facts
        .callable_bindings
        .iter()
        .any(|(_, name, id)| { name == "default" && *id == class_id }));
    assert!(facts
        .exported_bindings
        .iter()
        .any(|binding| { binding.local == "default" && binding.exported == "default" }));
}
