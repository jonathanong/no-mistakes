//! Partial canonical metadata must not invent a parameter effect owner.
use super::super::effects;
use super::collect_with_helpers;
use super::support::{assert_markers, fixture_with_effects};
use std::{path::PathBuf, sync::Arc};
fn saved_path() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads/effect-boundary-coverage.mts")
}
#[test]
fn imported_second_position_and_destructured_first_effects_stay_unproven() {
    let (source, _, heads) = fixture_with_effects("effect-boundary-coverage.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
    assert!(heads.exported_effects.contains_key("appendLocal"));
    assert!(!heads.exported_effects.contains_key("invalidFirst"));
    assert!(heads
        .pending_effects
        .iter()
        .all(|pending| pending.imported != "appendImported"));
}
#[test]
fn missing_or_duplicate_prepared_parameter_declarations_reject_effect_summary() {
    let (source, original, heads) = fixture_with_effects("effect-boundary-coverage.mts");
    assert!(heads.exported_effects.contains_key("appendLocal"));
    let calls = heads
        .calls
        .keys()
        .copied()
        .filter(|span| source[span.0 as usize..].starts_with("read("))
        .collect::<Vec<_>>();
    for duplicate in [false, true] {
        let mut ts = original.clone();
        let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
        let parameter_start = source.find("appendLocal(builder)").unwrap() + "appendLocal(".len();
        let parameter = module
            .bindings
            .iter()
            .find(|binding| binding.span.start as usize == parameter_start)
            .unwrap()
            .clone();
        if duplicate {
            module.bindings.push(parameter);
        } else {
            module.bindings.retain(|binding| binding.id != parameter.id);
        }
        let changed = crate::ast::with_program(&saved_path(), &source, |program, _| {
            collect_with_helpers(
                program,
                &ts,
                calls.iter().copied(),
                &Default::default(),
                true,
                true,
            )
        })
        .unwrap();
        assert!(!changed.exported_effects.contains_key("appendLocal"));
    }
}
#[test]
fn recovered_function_without_body_cannot_supply_an_effect_signature() {
    let source = std::fs::read_to_string(saved_path()).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let mut parsed =
        oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let invalid = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            let oxc_ast::ast::Statement::FunctionDeclaration(function) = statement else {
                return None;
            };
            (function.id.as_ref().unwrap().name == "invalidFirst").then_some(function)
        })
        .unwrap();
    assert!(effects::function(invalid).iter().all(Option::is_none));
    let function = parsed
        .program
        .body
        .iter_mut()
        .find_map(|statement| {
            let oxc_ast::ast::Statement::FunctionDeclaration(function) = statement else {
                return None;
            };
            (function.id.as_ref().unwrap().name == "recoveredBody").then_some(function)
        })
        .unwrap();
    assert!(effects::function(function)[0].is_some());
    // Recovery may retain a declaration while losing its body. Never invent purity.
    function.body = None;
    assert!(effects::function(function).iter().all(Option::is_none));
}
