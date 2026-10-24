use super::*;

#[test]
fn static_member_call_records_namespace_member_name() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "function run() { dates.parseDate(); }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .function_calls
        .iter()
        .any(|call| { call.caller.as_deref() == Some("run") && call.callee == "dates.parseDate" }));
}

#[test]
fn exported_declarations_record_identifier_and_type_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha } from './source.mts';\nimport type { SourceShape } from './source.mts';\nexport const alias: SourceShape = alpha;\nexport type AliasShape = SourceShape;",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts
        .symbol_references
        .iter()
        .any(|call| { call.caller.as_deref() == Some("alias") && call.callee == "alpha" }));
    assert!(facts
        .symbol_references
        .iter()
        .any(|call| { call.caller.as_deref() == Some("alias") && call.callee == "SourceShape" }));
    assert!(facts.symbol_references.iter().any(|call| {
        call.caller.as_deref() == Some("AliasShape") && call.callee == "SourceShape"
    }));
}

#[test]
fn function_parameters_shadow_imported_symbol_references() {
    let allocator = Allocator::default();
    let ret = Parser::new(
        &allocator,
        "import { alpha, beta, gamma, rest } from './source.mts';
         export function run({ alpha }: { alpha: number }, [beta]: number[], gamma = 1, ...rest: number[]) {
           return alpha + beta + gamma + rest.length;
         }",
        SourceType::ts(),
    )
    .parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let callees: Vec<_> = facts
        .symbol_references
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run"))
        .map(|call| call.callee.as_str())
        .collect();

    assert!(!callees.contains(&"alpha"));
    assert!(!callees.contains(&"beta"));
    assert!(!callees.contains(&"gamma"));
    assert!(!callees.contains(&"rest"));
}

#[test]
fn fixture_assignment_pattern_shadows_imported_calls() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/assignment-pattern-shadow.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert_eq!(
        calls.len(),
        1,
        "the shadowed runtime call remains observable"
    );
    assert_eq!(calls[0].target_identity, CallTargetIdentity::Unknown);
    assert!(!facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded"));
}

#[test]
fn fixture_anonymous_variable_function_records_anonymous_scope() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/anonymous-variable-function.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("<anonymous:1>")
    );
}

#[test]
fn fixture_catch_parameter_shadows_only_catch_block_references() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/catch-shadow.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert_eq!(calls.len(), 3);
}

#[test]
fn fixture_var_binding_shadows_outside_block_references() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/var-shadow.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);
    let calls: Vec<_> = facts
        .function_calls
        .iter()
        .filter(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded")
        .collect();

    assert_eq!(calls.len(), 1);
    assert!(!facts
        .symbol_references
        .iter()
        .any(|call| call.caller.as_deref() == Some("run") && call.callee == "loaded"));
}

#[test]
fn fixture_function_expression_falls_back_to_inner_name() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/destructured-function-expression.mts");
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(
        facts.imports[0].function_scope.as_deref(),
        Some("namedLoader")
    );
}

#[test]
fn local_string_named_export_is_not_marked_as_exported_function() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/local-string-named-export.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert!(facts.exported_functions.is_empty());
}

#[test]
fn exported_functions_are_sorted() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/exported-functions-order.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.exported_functions, vec!["alpha", "middle", "zeta"]);
}

#[test]
fn function_expression_declarator_binding_pattern_is_visited() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/function-expression-binding-pattern.mts",
    );
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, SourceType::ts()).parse();

    let facts = extract_import_facts_from_program(&ret.program);

    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].specifier, "./cfg.mts");
    assert_eq!(facts.imports[0].function_scope, None);
}
