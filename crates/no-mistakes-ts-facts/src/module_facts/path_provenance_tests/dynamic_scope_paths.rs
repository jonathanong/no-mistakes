use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

#[test]
fn dynamic_scope_forgets_an_imported_call_path() {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("dynamic-scope-import.mjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .with_check_syntax_error(true)
        .build(&parsed.program);
    let facts = super::extract_from_semantic(&built.semantic, &built.diagnostics, true);
    let (calls, provenance) = super::arguments::collect(&built.semantic, &facts);
    assert!(provenance.dynamic_scope);
    let offset = source.find("readFileSync(\"reports/a.txt\")").unwrap() as u32;
    assert_eq!(
        calls.get(&offset).and_then(|call| call.path.clone()),
        Some(PathOrigin::unknown())
    );
}
