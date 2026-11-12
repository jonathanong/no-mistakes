//! Closed local bindings whose every assigned value has a fresh literal head.
use super::{assignments::Context, bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
mod collect;
mod proofs;

pub(super) struct Root {
    tags: Vec<Span>,
}
pub(super) struct Initial {
    root: Root,
    span: Span,
    context: Context,
}
#[derive(Default)]
pub(super) struct Candidates {
    declarations: FxHashMap<Span, Initial>,
    writes: FxHashMap<Span, Root>,
}
impl Candidates {
    pub fn references(&self) -> impl Iterator<Item = Span> + '_ {
        self.declarations
            .values()
            .map(|initial| &initial.root)
            .chain(self.writes.values())
            .flat_map(|root| root.tags.iter().copied())
    }
}
pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> Vec<(Span, HeadProof)> {
    module.map_or_else(Vec::new, |module| {
        proofs::collect(syntax, bindings, module, executors)
    })
}
