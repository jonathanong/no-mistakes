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
            Some("handlers/method"),
            Some("handlers/expression"),
            Some("handlers/arrow"),
            Some("default")
        ]
    );
    assert!(facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("handlers/method")
            && call.callee == "SourceShape"));
}

#[test]
fn generic_type_parameters_shadow_imported_type_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import type { SourceShape } from './source.mts';\nexport type Box<SourceShape> = SourceShape;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.symbol_references, Vec::<FunctionCall>::new());
}

#[test]
fn fixture_default_function_expression_uses_expression_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/default-function-expression.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./loaded.mts");
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("<anonymous:1>")
    );
}

#[test]
fn fixture_nested_class_uses_default_walk() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/nested-class.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./loaded.mts");
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("outer/run")
    );
}

#[test]
fn fixture_local_enum_uses_default_walk() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/local-enum.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./source.mts");
    assert_eq!(facts.symbol_references.len(), 1);
    assert_eq!(facts.symbol_references[0].caller, None);
    assert_eq!(facts.symbol_references[0].callee, "alpha");
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
