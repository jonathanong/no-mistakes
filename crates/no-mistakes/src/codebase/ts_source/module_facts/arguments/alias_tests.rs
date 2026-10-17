use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

#[test]
fn popular_alias_scans_once_and_memoizes_budget_failure() {
    let path = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates")
        .join("popular-alias.mjs");
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let symbol = semantic
        .scoping()
        .symbol_ids()
        .find(|&s| semantic.scoping().symbol_name(s) == "command")
        .unwrap();
    assert_eq!(
        semantic.scoping().get_resolved_references(symbol).count(),
        300
    );
    let exported = FxHashSet::default();
    let mut immutable = AliasSafety::default();
    let mut projector = Projector {
        semantic: &semantic,
        call_offset: 0,
        remaining: 256,
        exported: &exported,
        immutable: &mut immutable,
    };
    for _ in 0..300 {
        assert!(!projector.immutable_binding(symbol));
    }
    assert_eq!(projector.immutable.scanned_references, 256);
    assert_eq!(projector.immutable.entries.len(), 1);
}
