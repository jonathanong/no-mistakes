use super::*;

#[test]
fn named_local_callable_arguments_record_callback_transitions() {
    let source = r#"
        async function helper() {
            await import("./reachable.mts");
        }
        export function run() {
            setTimeout(helper, 0);
        }
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.function_calls.iter().any(|call| {
        call.caller.as_deref() == Some("run")
            && call.callee == "helper"
            && call.is_callback
            && call.invocation == InvocationKind::Callback
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn constructor_callable_arguments_record_callback_transitions() {
    let source = r#"
        async function helper() {
            await import("./reachable.mts");
        }
        class Service {
            constructor(cb) {}
        }
        new Service(helper);
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert!(facts.function_calls.iter().any(|call| {
        call.callee == "helper"
            && call.is_callback
            && call.invocation == InvocationKind::Callback
            && call.target_identity == CallTargetIdentity::RepositoryFunction
    }));
}

#[test]
fn callable_type_parameters_collect_constraint_and_default_imports() {
    let source = r#"
        export function load<T extends import("./constraint.mts").Shape = import("./default.mts").Shape>() {}
        export const transform = <T extends import("./arrow-constraint.mts").Shape = import("./arrow-default.mts").Shape>() => {};
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let imports = facts
        .imports
        .iter()
        .map(|import| import.specifier.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        imports,
        vec![
            "./constraint.mts",
            "./default.mts",
            "./arrow-constraint.mts",
            "./arrow-default.mts",
        ]
    );
}

#[test]
fn typed_default_callable_expression_keeps_default_scope() {
    let source = r#"
        export default ((value: string) => import("./reachable.mts")) satisfies Loader;
    "#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].function_scope.as_deref(), Some("default"));
    assert!(facts.callable_scopes.contains(&"default".to_string()));
}
