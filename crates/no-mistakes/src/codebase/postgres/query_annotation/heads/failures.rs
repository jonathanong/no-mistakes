use super::support::{decisions, fixture, recollect};
use crate::codebase::ts_source::module_facts::ModuleDiagnostic;
use std::sync::Arc;

#[test]
fn missing_semantics_rejects_const_but_retains_original_literal_proofs() {
    let (source, mut ts, _) = fixture("root.mts");
    ts.module_bindings = None;
    let heads = recollect("root.mts", &ts);
    let calls = decisions(&source, &heads);
    assert!(calls.iter().any(|(span, accepted)| *accepted
        && source[span.0 as usize..].starts_with("read(\"/* direct */")));
    assert!(calls
        .iter()
        .filter(|(span, _)| source[span.0 as usize..].starts_with("read(named)"))
        .all(|(_, accepted)| !accepted));
}
#[test]
fn parse_or_operational_failure_never_emits_a_head_proof() {
    let (_, original, _) = fixture("root.mts");
    for failure in 0..3 {
        let mut ts = original.clone();
        match failure {
            0 => ts.parse_error = Some("saved parse failure".into()),
            1 => ts.operational_error = Some("saved operational failure".into()),
            _ => ts.fatal_parse_error = true,
        }
        assert!(recollect("root.mts", &ts).calls.is_empty());
    }
}
#[test]
fn duplicate_semantic_owners_fail_even_when_one_is_a_valid_const() {
    let (source, mut ts, _) = fixture("duplicate-owner.mts");
    let module = Arc::make_mut(ts.module_bindings.as_mut().unwrap());
    let mut other = module
        .bindings
        .iter()
        .find(|binding| binding.span.start == (source.find("const named").unwrap() + 6) as u32)
        .unwrap()
        .clone();
    assert!(!other.references.is_empty());
    other.imported = true;
    module.bindings.push(other);
    let heads = recollect("duplicate-owner.mts", &ts);
    let selected = decisions(&source, &heads)
        .into_iter()
        .filter(|(span, _)| source[span.0 as usize..].starts_with("read(named)"))
        .collect::<Vec<_>>();
    assert!(!selected.is_empty());
    assert!(selected.iter().all(|(_, accepted)| !accepted));
}
#[test]
fn semantic_error_without_location_cannot_authorize_lexical_const_ownership() {
    let (source, mut ts, _) = fixture("root.mts");
    Arc::make_mut(ts.module_bindings.as_mut().unwrap())
        .diagnostics
        .push(ModuleDiagnostic {
            kind: "semanticError".into(),
            message: "unlocated semantic ownership failure".into(),
            span: None,
        });
    let heads = recollect("root.mts", &ts);
    let selected = decisions(&source, &heads);
    assert!(selected.iter().any(|(_, accepted)| *accepted));
    let constants = selected
        .iter()
        .filter(|(span, _)| source[span.0 as usize..].starts_with("read(named)"))
        .collect::<Vec<_>>();
    assert!(!constants.is_empty());
    assert!(constants.iter().all(|(_, accepted)| !accepted));
}
