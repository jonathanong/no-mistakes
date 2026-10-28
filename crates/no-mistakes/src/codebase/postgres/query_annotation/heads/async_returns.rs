//! Bounded fresh native async return origins; no promise or SQL evaluation.
use super::{effects, HeadProof, Span};
mod caller;
mod producer;
mod projection;
mod resolve;
mod source;
pub(super) use caller::callers;
pub(super) use producer::function;
pub(super) use projection::collect;
pub(super) use resolve::project;

pub(super) struct Candidate {
    function: Span,
    body: Span,
    declaration: Span,
    tag: Span,
    returned: Span,
    initializer_start: u32,
    proof: HeadProof,
    appends: Vec<Span>,
    effects: Vec<Span>,
}
impl Candidate {
    pub(super) fn references(&self) -> impl Iterator<Item = Span> + '_ {
        [self.tag, self.returned]
            .into_iter()
            .chain(self.appends.iter().copied())
            .chain(self.effects.iter().copied())
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Summary {
    proof: HeadProof,
    required: Vec<effects::Pending>,
}
#[derive(Clone, Copy)]
pub(super) struct Caller {
    body: Span,
    declaration: Span,
    callee: Span,
    reference: Span,
    call: Span,
}
impl Caller {
    pub(super) fn references(&self) -> [Span; 2] {
        [self.callee, self.reference]
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    span: Span,
    specifier: String,
    imported: String,
}
