//! Exact paired local `{ text, values }` extraction from one helper result.
use super::{span, Syntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, VariableDeclarator};
use oxc_span::GetSpan;

#[derive(Clone, Copy)]
pub(in crate::codebase::postgres::query_annotation::heads) struct Destructure {
    pub text: super::super::Span,
    pub values: super::super::Span,
    pub callee: super::super::Span,
    pub init: super::super::Span,
    pub context: super::super::assignments::Context,
}

impl Syntax {
    pub(super) fn record_destructure(&mut self, declaration: &VariableDeclarator<'_>) {
        let BindingPattern::ObjectPattern(object) = &declaration.id else {
            return;
        };
        if object.rest.is_some() || object.properties.len() != 2 {
            return;
        }
        let Some(init) = &declaration.init else {
            return;
        };
        let Some(callee) = super::super::helper_heads::callee(init) else {
            return;
        };
        let mut text = None;
        let mut values = None;
        for property in &object.properties {
            if property.computed || !property.shorthand {
                return;
            }
            let BindingPattern::BindingIdentifier(local) = &property.value else {
                return;
            };
            let target = match crate::ast::property_key_name(&property.key).as_deref() {
                Some("text") => &mut text,
                Some("values") => &mut values,
                _ => return,
            };
            if target.replace(span(local.span)).is_some() {
                return;
            }
        }
        let (Some(text), Some(values)) = (text, values) else {
            return;
        };
        self.contexts.insert(text, self.current_function);
        self.contexts.insert(values, self.current_function);
        self.record_destructures.insert(
            text,
            Destructure {
                text,
                values,
                callee,
                init: span(init.span()),
                context: self.assignments.context,
            },
        );
    }

    pub(super) fn destructure_call(&mut self, call: &CallExpression<'_>) {
        if call.optional || call.arguments.len() != 2 {
            return;
        }
        let (Some(first), Some(second)) = (
            call.arguments.first().and_then(|arg| arg.as_expression()),
            call.arguments.get(1).and_then(|arg| arg.as_expression()),
        ) else {
            return;
        };
        let (Expression::Identifier(text), Expression::Identifier(values)) =
            (unwrap_ts_wrappers(first), unwrap_ts_wrappers(second))
        else {
            return;
        };
        self.destructure_calls
            .insert(span(call.span), (span(text.span), span(values.span)));
    }
}
