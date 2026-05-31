use super::*;

#[test]
fn inline_function_like_members_record_parameters_and_type_parameters() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { loaded, SourceShape } from './source.mts';
         const handlers = {
           method<T extends SourceShape>(value: T = loaded()) { return value; },
           expression: function<T extends SourceShape>(value: T = loaded()) { return value; },
           arrow: <T extends SourceShape>(value: T = loaded()) => value,
         };
         export default function<T extends SourceShape>(value: T = loaded()) { return value; }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let scopes: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.callee == "loaded")
        .map(|call| call.caller.as_deref())
        .collect();

    assert_eq!(
        scopes,
        vec![
            Some("method"),
            Some("expression"),
            Some("arrow"),
            Some("default")
        ]
    );
    assert!(facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("method") && call.callee == "SourceShape"));
}

#[test]
fn collector_defensive_scope_helpers_are_noops_without_active_scope() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "function typed<T>() {}\nconst { value = fallback } = input;",
        SourceType::ts(),
    )
    .parse();

    let type_params = ret
        .program
        .body
        .iter()
        .find_map(|statement| match statement {
            oxc::ast::ast::Statement::FunctionDeclaration(function) => {
                function.type_parameters.as_deref()
            }
            _ => None,
        })
        .expect("fixture function should have type parameters");
    let binding = ret
        .program
        .body
        .iter()
        .find_map(|statement| match statement {
            oxc::ast::ast::Statement::VariableDeclaration(declaration) => declaration
                .declarations
                .first()
                .map(|declarator| &declarator.id),
            _ => None,
        })
        .expect("fixture variable should have a binding pattern");
    let mut collector = ImportCollector::default();

    collector.add_type_parameter_names(Some(type_params));
    collector.add_function_binding_names(binding);
    collector.add_binding_names(binding);
    collector.add_binding_name("value");
    collector.known_function_scopes.insert("known".to_string());

    assert!(collector.has_local_function_scope("known"));
    assert_eq!(binding_names(binding), vec!["value"]);
}
