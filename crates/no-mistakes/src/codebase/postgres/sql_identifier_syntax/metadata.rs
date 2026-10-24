//! Later executor arguments are metadata, scoped to each projected call owner.
use super::*;
type Context = (Option<Span>, Option<Span>);
#[derive(Default)]
pub(super) struct Metadata {
    call: Option<Span>,
    argument: Option<Span>,
    references: FxHashMap<Span, Span>,
}
impl Metadata {
    pub fn enter_call(&mut self, call: Option<Span>) -> Context {
        let previous = (self.call, self.argument);
        (self.call, self.argument) = (call, None);
        previous
    }
    pub fn argument(&mut self, index: usize) -> Context {
        let previous = (self.call, self.argument);
        self.argument = (index > 0).then_some(self.call).flatten();
        previous
    }
    pub fn restore(&mut self, previous: Context) {
        (self.call, self.argument) = previous;
    }
    pub fn reference(&mut self, at: Span) {
        if let Some(call) = self.argument {
            self.references.insert(at, call);
        }
    }
    pub fn belongs_to(&self, at: &Span, direct: &FxHashSet<Span>) -> bool {
        self.references
            .get(at)
            .is_some_and(|call| direct.contains(call))
    }
}
impl Syntax {
    // Opaque invocations and deferred bodies own their uses independently of
    // the surrounding executor's metadata; those escapes remain unknown.
    pub(super) fn opaque_metadata(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.metadata.enter_call(None);
        visit(self);
        self.metadata.restore(previous);
    }
}
