//! Original syntax proofs for annotation low effort; no helper value evaluation.
use std::collections::BTreeMap;
mod append;
mod append_constants;
mod assignments;
mod bindings;
mod callback_heads;
mod captured_prefix;
mod collect;
mod conditional;
mod diagnostics;
mod effects;
mod finite_comments;
mod finite_returns;
mod helper_heads;
mod helper_records;
mod import_projection;
mod literal_helpers;
mod lookup;
mod loop_carried;
mod loop_word;
mod namespace_heads;
pub(crate) use loop_word::{prepare_loop_heads, PreparedLoopHeads};
mod syntax;
mod tag_constants;
#[cfg(test)]
mod tests;
mod text;
mod trusted_append;
#[cfg(test)]
mod trusted_append_tests;
pub(super) type Span = (u32, u32);
/// Provisional executor calls supply helper heads but never safe builder uses.
#[derive(Default)]
pub(crate) struct HeadCallSpans {
    pub(crate) confirmed: crate::fx::FxHashSet<Span>,
    pub(crate) pending: crate::fx::FxHashSet<Span>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadProof {
    Literal,
    Missing,
    Unproven,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct AnnotationHeads {
    pub(crate) calls: BTreeMap<Span, HeadProof>,
    pub(crate) exported_helpers: BTreeMap<String, HeadProof>,
    pub(crate) exported_values: BTreeMap<String, HeadProof>,
    pub(crate) exported_records: BTreeMap<String, helper_records::Summary>,
    pub(crate) pending_imports: Vec<helper_heads::Pending>,
    pub(crate) pending_namespaces: Vec<namespace_heads::Pending>,
    pub(crate) exported_effects: BTreeMap<String, effects::Signature>,
    pub(crate) pending_effects: Vec<effects::Pending>,
    pub(crate) exported_callbacks: BTreeMap<String, callback_heads::Summary>,
    pub(crate) pending_callbacks: Vec<callback_heads::Pending>,
    pub(crate) conditional_effects: Vec<effects::Conditional>,
    pub(crate) loop_carried_missing: BTreeMap<Span, Vec<Option<String>>>,
    helper_calls: Option<BTreeMap<Span, (String, HeadProof)>>,
    pub(crate) trusted_append_calls: crate::fx::FxHashSet<Span>,
}
pub(crate) use collect::collect_prepared;

pub(crate) use import_projection::project;

#[cfg(test)]
mod literal_helper_tests;
