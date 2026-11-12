use super::*;
use crate::codebase::ts_source::module_facts::arguments::{aliases::AliasSafety, path_origin};
use crate::codebase::ts_source::module_facts::extract_from_semantic;
use crate::fx::FxHashSet;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;

fn with_projector(test: impl FnOnce(&mut Projector<'_, '_>)) {
    let path = crate::test_support::rule_fixture_root("test-no-live-repository-scan/facts")
        .join("callback-uses.mts");
    let source_type = SourceType::from_path(&path).unwrap();
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let semantic = built.semantic;
    let facts = extract_from_semantic(&semantic, &[], false);
    let temp_factories = path_origin::temp_factories(&semantic, &facts);
    let exported = FxHashSet::default();
    let mut immutable = AliasSafety::default();
    let mut projector = Projector {
        semantic: &semantic,
        call_offset: 0,
        remaining: 256,
        exported: &exported,
        immutable: &mut immutable,
        temp_factories: &temp_factories,
    };
    test(&mut projector);
}

fn symbol(semantic: &Semantic<'_>, name: &str) -> SymbolId {
    semantic
        .scoping()
        .symbol_ids()
        .find(|&symbol| semantic.scoping().symbol_name(symbol) == name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn root_uses_require_remaining_budget_and_read_only_local_references() {
    for (root, callback, helper, budget, expected) in [
        (
            "budgetRoot",
            "budgetCallback",
            "budgetHelper",
            256,
            Some(()),
        ),
        ("budgetRoot", "budgetCallback", "budgetHelper", 0, None),
        ("writtenRoot", "writeCallback", "writeHelper", 256, None),
        (
            "capturedRoot",
            "closureCallback",
            "closureHelper",
            256,
            None,
        ),
    ] {
        with_projector(|projector| {
            let root = symbol(projector.semantic, root);
            let callback = symbol(projector.semantic, callback);
            let helper = symbol(projector.semantic, helper);
            let helper = projector
                .semantic
                .nodes()
                .kind(projector.semantic.scoping().symbol_declaration(helper))
                .span()
                .start;
            projector.remaining = budget;
            assert_eq!(
                projector.callback_root_uses(root, callback, false, helper),
                expected
            );
        });
    }
}
