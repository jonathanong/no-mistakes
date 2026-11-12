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
        .join("callback-classification.mts");
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

fn declaration<'a>(semantic: &Semantic<'a>, name: &str) -> AstKind<'a> {
    semantic.nodes().kind(
        semantic
            .scoping()
            .symbol_declaration(symbol(semantic, name)),
    )
}

#[test]
fn temporary_arguments_follow_each_static_helper_declaration_shape() {
    for helper in [
        "arrowHelper",
        "expressionHelper",
        "declaredHelper",
        "aliasHelper",
    ] {
        with_projector(|projector| {
            let kind = declaration(projector.semantic, helper);
            assert!(projector.callback_temp_argument(kind, 0, 0), "{helper}");
        });
    }
}

#[test]
fn unsupported_helpers_and_parameter_shapes_do_not_classify_as_temporary() {
    for helper in [
        "scalarHelper",
        "absentHelper",
        "destructuredHelper",
        "unusedHelper",
        "repositoryHelper",
    ] {
        with_projector(|projector| {
            let kind = declaration(projector.semantic, helper);
            assert!(!projector.callback_temp_argument(kind, 0, 0), "{helper}");
        });
    }
    with_projector(|projector| {
        let kind = declaration(projector.semantic, "declaredHelper");
        assert!(!projector.callback_temp_argument(kind, 1, 0));
        let kind = declaration(projector.semantic, "declaredCallback");
        assert!(!projector.callback_temp_argument(kind, 0, 0));
    });
}

#[test]
fn type_references_destructuring_and_calls_without_an_argument_are_skipped() {
    with_projector(|projector| {
        let callback = symbol(projector.semantic, "ignoredCallback");
        assert!(projector
            .semantic
            .scoping()
            .get_resolved_references(callback)
            .any(|reference| reference.is_type()));
        let kind = declaration(projector.semantic, "ignoredHelper");
        assert!(!projector.callback_temp_argument(kind, 0, 0));
    });
}

#[test]
fn alias_cycles_and_long_chains_stop_conservatively_at_the_depth_bound() {
    for helper in ["cycleHelper", "depthHelper"] {
        with_projector(|projector| {
            let kind = declaration(projector.semantic, helper);
            assert!(projector.callback_temp_argument(kind, 0, 0), "{helper}");
            assert!(projector.remaining > 0, "{helper}");
        });
    }
    with_projector(|projector| {
        let kind = declaration(projector.semantic, "shortHelper");
        assert!(!projector.callback_temp_argument(kind, 0, 0));
    });
}

#[test]
fn exhausted_reference_budget_is_conservative_and_never_underflows() {
    for remaining in [0, 1] {
        with_projector(|projector| {
            projector.remaining = remaining;
            let kind = declaration(projector.semantic, "shortHelper");
            assert!(projector.callback_temp_argument(kind, 0, 0));
            assert_eq!(projector.remaining, 0);
        });
    }
}
