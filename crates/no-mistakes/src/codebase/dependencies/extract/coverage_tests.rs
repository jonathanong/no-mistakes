use super::*;
use oxc_parser::Parser;

#[test]
fn top_level_type_bindings_shadow_file_fallback_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "type Local = { value: string };\ntype Public = Local;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    let refs: Vec<_> = facts
        .symbol_references
        .iter()
        .map(|reference| (reference.caller.as_deref(), reference.callee.as_str()))
        .collect();
    assert_eq!(refs, vec![(Some("Public"), "Local"), (None, "Local")]);
}

#[test]
fn nested_jsx_member_references_are_recorded() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { UI } from './source.mts';\nexport const view = <UI.Form.Input />;",
        SourceType::tsx(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .symbol_references
        .iter()
        .any(|reference| reference.callee == "UI.Form.Input"));
}

#[test]
fn nested_type_binding_can_resolve_to_known_function_scope() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "export function outer() {
           function Local() {}
           interface Local { value: string }
           type Uses = Local.Member;
         }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("outer") && reference.callee == "outer/Local.Member"
    }));
}

#[test]
fn parenthesized_default_object_expression_records_members() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default (({ method() { return alpha; } }));",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.symbol_references.iter().any(|reference| {
        reference.caller.as_deref() == Some("default/method") && reference.callee == "alpha"
    }));
}

#[test]
fn default_arrow_object_and_rest_params_are_collected() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default () => { alpha(); }",
        SourceType::ts(),
    )
    .parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("default") && call.callee == "alpha"));

    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nexport default { run() { alpha(); } };",
        SourceType::ts(),
    )
    .parse();
    let facts = extract_import_facts_from_program(&ret.program);
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.caller.as_deref() == Some("default") && call.callee == "run"));

    let ret = Parser::new(
        &allocator,
        "function outer(...rest: string[]) { function inner() { rest; } inner(); }",
        SourceType::ts(),
    )
    .parse();
    assert!(extract_import_facts_from_program(&ret.program)
        .imports
        .is_empty());
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
            oxc_ast::ast::Statement::FunctionDeclaration(function) => {
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
            oxc_ast::ast::Statement::VariableDeclaration(declaration) => declaration
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
