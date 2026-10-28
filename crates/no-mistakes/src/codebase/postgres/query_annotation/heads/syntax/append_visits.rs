//! Append metadata shares the assignment-aware syntax walk.
use super::{span, Syntax};
use oxc_ast::ast::{BindingIdentifier, CallExpression, Expression, IdentifierReference};

impl Syntax {
    pub(super) fn record_reference(&mut self, value: &IdentifierReference<'_>) {
        self.contexts
            .insert(span(value.span), self.current_function);
        if self.collect_helper_heads && (value.name == "arguments" || value.name == "eval") {
            self.dynamic_refs.push(span(value.span));
        }
    }
    pub(super) fn append_call(&mut self, value: &CallExpression<'_>) {
        if self.collect_helper_heads {
            self.self_appends.call(value);
        }
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
            if self.assignments.context.repeated.is_some() {
                self.repeated_append_refs.insert(span(id.span));
            }
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
