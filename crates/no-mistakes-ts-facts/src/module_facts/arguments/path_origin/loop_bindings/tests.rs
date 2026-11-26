use super::*;
use crate::fx::FxHashSet;
use crate::module_facts::arguments::aliases::AliasSafety;
use crate::module_facts::arguments::path_origin::TempFactories;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::path::PathBuf;

fn with_projector(test: impl FnOnce(&mut Projector<'_, '_>)) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/typescript-module-facts/owned-loop-bindings.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let mut immutable = AliasSafety::default();
    let exported = FxHashSet::default();
    let temp_factories = TempFactories::default();
    let mut projector = Projector {
        semantic: &built.semantic,
        call_offset: 0,
        remaining: 256,
        exported: &exported,
        immutable: &mut immutable,
        temp_factories: &temp_factories,
    };
    test(&mut projector);
}

fn identifier<'a>(projector: &Projector<'a, '_>, name: &str) -> &'a IdentifierReference<'a> {
    projector
        .semantic
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            AstKind::IdentifierReference(id) if id.name == name => Some(id),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing identifier {name}"))
}

#[test]
fn finite_loop_projection_needs_budget_for_every_literal_alternative() {
    with_projector(|projector| {
        let child = identifier(projector, "child");
        for budget in [0, 1, 2] {
            projector.remaining = budget;
            assert_eq!(projector.loop_binding(child), None, "budget {budget}");
            assert_eq!(projector.remaining, 0, "budget {budget}");
        }
        projector.remaining = 3;
        let origin = projector.loop_binding(child).unwrap();
        let alternatives: Vec<_> = origin
            .atoms()
            .iter()
            .map(|atom| (atom.anchor, atom.segments.clone(), atom.dynamic))
            .collect();
        assert_eq!(
            alternatives,
            vec![
                (
                    super::super::PathAnchor::Relative,
                    vec!["a".to_owned()],
                    false
                ),
                (
                    super::super::PathAnchor::Relative,
                    vec!["b".to_owned()],
                    false
                ),
            ]
        );
        assert_eq!(projector.remaining, 0);
    });
}

#[test]
fn unsupported_binding_declarations_never_supply_finite_loop_values() {
    with_projector(|projector| {
        for name in [
            "ordinaryFunction",
            "parameter",
            "outside",
            "resource",
            "mutable",
            "destructured",
        ] {
            projector.remaining = 256;
            let id = identifier(projector, name);
            assert_eq!(projector.loop_binding(id), None, "{name}");
        }
    });
}
