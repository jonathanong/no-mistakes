use super::super::{AnnotationHeads, HeadProof};
use super::support::{fixture, recollect};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, ModuleSpan};
use std::sync::Arc;

const FIXTURE: &str = "assignment-gaps.mts";

fn proof(heads: &AnnotationHeads) -> &HeadProof {
    assert_eq!(heads.calls.len(), 1);
    heads.calls.values().next().unwrap()
}

fn inject_gap(
    mut facts: crate::codebase::ts_source::facts::TsFileFacts,
    gap: ModuleSpan,
) -> crate::codebase::ts_source::facts::TsFileFacts {
    Arc::make_mut(facts.module_bindings.as_mut().unwrap())
        .diagnostics
        .push(ModuleDiagnostic {
            kind: "semanticError".into(),
            message: "injected ownership gap".into(),
            span: Some(gap),
        });
    facts
}

#[test]
fn declaration_reference_and_write_gaps_make_assignment_proofs_unproven() {
    let (source, facts, heads) = fixture(FIXTURE);
    assert!(matches!(proof(&heads), HeadProof::Literal));

    let binding = facts
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| binding.name == "query")
        .unwrap();
    let declaration = binding.span;
    let literal_start = source.find("/* reassigned */").unwrap() as u32;
    let assignment_gap = ModuleSpan {
        start: literal_start,
        end: literal_start + 1,
    };
    let read_start = source.rfind("read(query)").unwrap() as u32 + "read(".len() as u32;
    let reference_gap = binding
        .references
        .iter()
        .find(|reference| reference.span.start == read_start)
        .unwrap()
        .span;

    // This RHS gap intersects the full write expression without intersecting
    // its LHS reference, so it reaches classify()'s defensive Unknown outcome.
    for gap in [declaration, reference_gap, assignment_gap] {
        let heads = recollect(FIXTURE, &inject_gap(facts.clone(), gap));
        assert!(matches!(proof(&heads), HeadProof::Unproven), "gap: {gap:?}");
    }
}
