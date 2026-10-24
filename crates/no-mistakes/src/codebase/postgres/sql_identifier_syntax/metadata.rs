//! Distinguish known SQL data operands from opaque runtime executor escapes.
use super::*;
#[derive(Clone, Copy, Default)]
pub(super) struct Context {
    call: Option<Span>,
    argument: Option<Span>,
    tag: Option<usize>,
    bind: Option<usize>,
}
enum Owner {
    Call(Span),
    Bind(usize),
}
#[derive(Default)]
pub(super) struct Metadata {
    context: Context,
    references: FxHashMap<Span, Owner>,
}
impl Metadata {
    pub fn enter_call(&mut self, call: Option<Span>) -> Context {
        std::mem::replace(
            &mut self.context,
            Context {
                call,
                ..Default::default()
            },
        )
    }
    pub fn enter_tag(&mut self, tag: Option<usize>) -> Context {
        std::mem::replace(
            &mut self.context,
            Context {
                tag,
                ..Default::default()
            },
        )
    }
    pub fn bind_expression(&mut self) -> Context {
        let previous = self.context;
        if let Some(tag) = self.context.tag {
            self.context.bind = Some(tag);
        }
        previous
    }
    pub fn argument(&mut self, index: usize) -> Context {
        let previous = self.context;
        self.context.argument = (index > 0).then_some(self.context.call).flatten();
        previous
    }
    pub fn restore(&mut self, previous: Context) {
        self.context = previous;
    }
    pub fn reference(&mut self, at: Span) {
        let owner = self
            .context
            .argument
            .map(Owner::Call)
            .or_else(|| self.context.bind.map(Owner::Bind));
        if let Some(owner) = owner {
            self.references.insert(at, owner);
        }
    }
    pub fn belongs_to(
        &self,
        at: &Span,
        direct: &FxHashSet<Span>,
        tags: &template_binds::TemplateBinds,
    ) -> bool {
        match self.references.get(at) {
            Some(Owner::Call(call)) => direct.contains(call),
            // A later tag write invalidates bind classification request-wide.
            Some(Owner::Bind(id)) => tags.is_valid(*id),
            None => false,
        }
    }
}
impl Syntax {
    // Opaque invocations and deferred bodies own their uses independently of
    // the surrounding executor's data; those escapes remain unknown.
    pub(super) fn opaque_metadata(&mut self, visit: impl FnOnce(&mut Self)) {
        let previous = self.metadata.enter_call(None);
        visit(self);
        self.metadata.restore(previous);
    }
}
