use super::*;
use oxc_parser::Parser;

#[test]
fn finalized_callable_catalogs_share_repeated_scope_names() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/shared-scope-names.mts");
    let source = std::fs::read_to_string(&fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);
    assert_eq!(
        facts.known_function_scopes.capacity(),
        facts.known_function_scopes.len()
    );
    assert_eq!(
        facts.callable_scopes.capacity(),
        facts.callable_scopes.len()
    );
    assert_eq!(
        facts.callable_scope_ids.capacity(),
        facts.callable_scope_ids.len()
    );
    let scopes = facts
        .known_function_scopes
        .iter()
        .chain(&facts.callable_scopes)
        .chain(facts.callable_scope_ids.iter().map(|(_, scope)| scope))
        .collect::<Vec<_>>();
    let mut pointers = fx_map();
    for scope in &scopes {
        // Value equality alone would also pass with independently allocated Strings.
        if let Some(pointer) = pointers.insert(&***scope, scope.as_ptr()) {
            assert_eq!(
                pointer,
                scope.as_ptr(),
                "scope {scope} must share its allocation"
            );
        }
    }
    assert!(pointers.len() < scopes.len());
    assert!(pointers.contains_key("outer/nested"));
    assert!(pointers.contains_key("Worker/run/nested"));
    assert_eq!(facts.imports[0].specifier, "./shared-scope-executor.mjs");
    assert!(facts
        .function_calls
        .iter()
        .any(|call| call.callee == "nested"));
}

#[test]
fn finalized_scope_interner_is_not_retained_between_collections() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/shared-scope-names.mts");
    let source = std::fs::read_to_string(&fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source(&parsed.program, &source);
    let again = extract_import_facts_from_program_with_source(&parsed.program, &source);
    assert_eq!(facts, again);
    assert!(!std::sync::Arc::ptr_eq(
        &facts.known_function_scopes[0],
        &again.known_function_scopes[0]
    ));
    let scopes = facts
        .known_function_scopes
        .iter()
        .chain(&facts.callable_scopes)
        .chain(facts.callable_scope_ids.iter().map(|(_, scope)| scope))
        .collect::<Vec<_>>();
    let mut counts = fx_map();
    for scope in &scopes {
        *counts.entry(&***scope).or_insert(0) += 1;
    }
    for scope in scopes {
        assert_eq!(
            std::sync::Arc::strong_count(scope),
            counts[&**scope],
            "the finalized facts must own every reference; the interner must be dropped"
        );
    }
}
