use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashSet;
use std::collections::BTreeMap;

pub(super) fn apply(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    ts: &TsFileFacts,
    executors: &FxHashSet<Span>,
    calls: &mut BTreeMap<Span, HeadProof>,
) {
    for (span, proof) in super::string_prefix::proofs(syntax, bindings, executors) {
        if matches!(calls.get(&span), Some(HeadProof::Unproven)) {
            calls.insert(span, proof);
        }
    }
    for (span, proof) in super::branch_assignments::proofs(syntax, bindings, executors)
        .into_iter()
        .chain(super::fresh_builders::proofs(
            syntax,
            bindings,
            ts.module_bindings.as_deref(),
            executors,
        ))
    {
        if matches!(calls.get(&span), Some(HeadProof::Unproven)) {
            calls.insert(span, proof);
        }
    }
}
