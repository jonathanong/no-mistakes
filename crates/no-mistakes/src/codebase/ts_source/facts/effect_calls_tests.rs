use super::effect_calls::*;
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::Path;

#[test]
fn preserves_terminal_effects_on_non_identifier_receivers() {
    let source = "this.invalidate(); factory().invalidate();";
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        Path::new("non-identifier-effects.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    let facts = extract_import_facts_from_program_with_source(&parsed.program, source);
    let effects = collect_effect_calls(
        &facts.function_calls,
        &EffectNames::from([("invalidate".to_string(), Some("cache".to_string()))]),
    );

    assert_eq!(effects.len(), 2);
    assert!(effects.iter().all(|effect| effect.callee == "invalidate"));
    assert!(effects
        .iter()
        .all(|effect| effect.category.as_deref() == Some("cache")));
}
