//! Append metadata shares the assignment-aware syntax walk.
use super::{span, Syntax};
use oxc_ast::ast::{BindingIdentifier, CallExpression, Expression};

impl Syntax {
    pub(super) fn append_call(&mut self, value: &CallExpression<'_>) {
        if self.collect_append_tags {
            if let Some(tag) = value
                .arguments
                .first()
                .and_then(|arg| arg.as_expression())
                .and_then(super::super::trusted_append::tag)
            {
                self.append_tags.insert(span(value.span), tag);
            }
        }
        if let Some(Expression::Identifier(id)) = super::super::append::receiver(value)
            .map(crate::codebase::ts_source::unwrap_ts_wrappers)
        {
            self.append_receivers.insert(span(id.span));
        }
        if self.assignments.context.repeated.is_some() {
            self.repeated_calls.insert(span(value.span));
        }
    }
    pub(super) fn append_declaration<'a>(
        &mut self,
        binding: &BindingIdentifier<'a>,
        init: Option<&Expression<'a>>,
    ) {
        self.contexts
            .insert(span(binding.span), self.current_function);
        if self.collect_append_tags {
            if let Some(tag) = init.and_then(super::super::trusted_append::tag) {
                self.append_tags.insert(span(binding.span), tag);
            }
        }
    }
}
