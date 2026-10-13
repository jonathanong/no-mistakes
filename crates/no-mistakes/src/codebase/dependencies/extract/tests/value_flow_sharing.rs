use super::*;
use oxc_parser::Parser;

#[test]
fn destructured_value_bindings_share_one_initializer() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts");
    let source = std::fs::read_to_string(fixture).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    let import_only = extract_import_facts_from_program(&parsed.program);
    assert!(import_only.value_flow.bindings.is_empty());
    assert!(import_only.value_flow.parameters.is_empty());
    assert!(import_only.value_flow.calls.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program, &source, false, false, true,
    );
    let bases = facts
        .value_flow
        .bindings
        .iter()
        .filter_map(|binding| match &binding.value {
            ValueExpression::Member(base, _) => Some(base),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bases.len(), 128);
    assert!(bases
        .iter()
        .all(|base| std::sync::Arc::ptr_eq(bases[0], base)));
    assert!(matches!(bases[0].as_ref(), ValueExpression::Object(fields) if fields.len() == 128));
}
