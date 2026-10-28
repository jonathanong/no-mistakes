use super::super::HeadProof;
use super::support::{assert_markers, fixture, recollect};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, ModuleSpan};
use std::sync::Arc;

#[test]
fn primitive_strings_keep_only_closed_continuation_safe_heads() {
    let (source, _, heads) = fixture("string-prefix.mts");
    assert_eq!(assert_markers(&source, &heads), 28);
    let (source, _, heads) = fixture("string-prefix-eval.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn every_initializer_write_and_reference_gap_rejects_the_string_proof() {
    let name = "string-prefix-cache.mts";
    let (source, facts, heads) = fixture(name);
    assert_eq!(
        heads
            .calls
            .values()
            .filter(|&&p| p == HeadProof::Literal)
            .count(),
        3
    );
    for needle in ["/* accepted */", "sql +=", "read(sql)"] {
        let start = source.find(needle).unwrap() as u32;
        let mut facts = facts.clone();
        Arc::make_mut(facts.module_bindings.as_mut().unwrap())
            .diagnostics
            .push(ModuleDiagnostic {
                kind: "semanticError".into(),
                message: "injected ownership gap".into(),
                span: Some(ModuleSpan {
                    start,
                    end: start + needle.len() as u32,
                }),
            });
        let heads = recollect(name, &facts);
        assert!(
            heads.calls.values().all(|p| *p != HeadProof::Literal),
            "{needle}"
        );
    }
}

#[test]
fn duplicate_semantic_owners_cannot_establish_a_string_prefix() {
    let name = "string-prefix-cache.mts";
    let (_, mut facts, _) = fixture(name);
    let module = Arc::make_mut(facts.module_bindings.as_mut().unwrap());
    let binding = module
        .bindings
        .iter()
        .find(|b| b.name == "sql")
        .unwrap()
        .clone();
    module.bindings.push(binding);
    let heads = recollect(name, &facts);
    assert!(heads.calls.values().all(|p| *p != HeadProof::Literal));
}
