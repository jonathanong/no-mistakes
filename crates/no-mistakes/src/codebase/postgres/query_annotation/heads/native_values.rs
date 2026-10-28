//! Values observations preserve only a closed native statement's SQL head.
//! No value prediction, bounds, purity or standard `.at` implementation is assumed.
use super::Span;
mod facts;
mod ownership;
mod proof;
pub(super) use facts::Facts;
pub(super) use ownership::extend_wanted;
pub(super) use proof::proofs;

pub(super) fn apply(
    syntax: &super::syntax::Syntax,
    bindings: &super::bindings::Bindings<'_>,
    ts: &crate::codebase::ts_source::facts::TsFileFacts,
    executors: &crate::fx::FxHashSet<Span>,
    calls: &mut std::collections::BTreeMap<Span, super::HeadProof>,
) {
    for (span, proof) in proofs(syntax, bindings, ts.module_bindings.as_deref(), executors) {
        if matches!(calls.get(&span), Some(super::HeadProof::Unproven)) {
            calls.insert(span, proof);
        }
    }
}
