use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn generated_hex_identifiers_require_literal_shapes_and_unshadowed_crypto_imports() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/codebase-analysis/generated-sql-identifiers/facts.ts"
    ));
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        true,
    );
    let mut names: Vec<_> = facts
        .value_flow
        .bounded_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "aliasName",
            "defaultName",
            "directName",
            "namespaceName",
            "scopedName",
            "suffixName",
            "wrappedName",
            "zeroName",
        ]
    );
}
