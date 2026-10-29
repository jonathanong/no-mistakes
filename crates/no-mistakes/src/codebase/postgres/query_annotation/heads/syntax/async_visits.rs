//! Original visitor facts for the bounded async origin proof.
use super::{span, Syntax};
use oxc_ast::ast::Statement;
impl Syntax {
    pub(super) fn async_body(&mut self, statements: &[Statement<'_>], body: oxc_span::Span) {
        if self.collect_helper_heads {
            self.async_callers
                .extend(super::super::async_returns::callers(statements, body));
        }
    }
    pub(super) fn async_await(&mut self, value: oxc_span::Span) {
        if self.collect_helper_heads && !self.async_returns.is_empty() {
            self.async_awaits.push(span(value));
        }
    }
}
