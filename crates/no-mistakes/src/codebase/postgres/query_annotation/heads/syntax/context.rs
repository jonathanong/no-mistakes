use super::Syntax;
use crate::codebase::postgres::query_annotation::heads::assignments::Context;
impl Syntax {
    pub(super) fn context(&mut self, context: Context, walk: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.assignments.context, context);
        walk(self);
        self.assignments.context = previous;
    }
    pub(super) fn repeating(&mut self, span: oxc_span::Span, walk: impl FnOnce(&mut Self)) {
        let mut context = self.assignments.context;
        context.repeated.get_or_insert((span.start, span.end));
        self.context(context, walk);
    }
}
