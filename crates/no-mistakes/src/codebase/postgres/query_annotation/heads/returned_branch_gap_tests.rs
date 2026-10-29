//! One unresolved branch gap must never queue a partial native result proof.
use super::support::{fixture_with_effects, recollect};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, ModuleSpan};
use std::sync::Arc;
#[test]
fn returned_parameter_branch_gap_rejects_the_complete_projection() {
    let name = "returned-param-branch-gap.mts";
    let (source, mut facts, heads) = fixture_with_effects(name);
    assert_eq!(heads.pending_returned_params.len(), 1);
    let start = source.find("/* two */").unwrap() as u32;
    Arc::make_mut(facts.module_bindings.as_mut().unwrap())
        .diagnostics
        .push(ModuleDiagnostic {
            kind: "semanticError".into(),
            message: "injected branch ownership gap".into(),
            span: Some(ModuleSpan {
                start,
                end: start + 9,
            }),
        });
    let changed = recollect(name, &facts);
    assert!(changed.pending_returned_params.is_empty());
}
