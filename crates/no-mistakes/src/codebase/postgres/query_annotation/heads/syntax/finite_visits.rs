//! Candidate facts recorded during the one original syntax visit.
use super::{span, Syntax};
use crate::codebase::postgres::query_annotation::heads::finite_comments::{
    self, Argument, CallSite, Function, Select, TemplateUse,
};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, CallExpression, Expression, Function as AstFunction,
    FunctionType, VariableDeclarator,
};

impl Syntax {
    pub(super) fn finite_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_finite {
            return;
        }
        if let Some(expression) = call.arguments.first().and_then(|arg| arg.as_expression()) {
            if let Some((reference, before, after)) = finite_comments::template(expression) {
                if let Some(domain) = self.active_domains.iter().rev().find(|domain| {
                    matches!(expression,
                        Expression::TemplateLiteral(value) if value.expressions.first()
                            .is_some_and(|part| matches!(part, Expression::Identifier(id)
                                if id.name.as_str() == domain.name)))
                }) {
                    self.finite_templates.push(TemplateUse {
                        call: span(call.span),
                        reference,
                        domain: domain.clone(),
                        before,
                        after,
                    });
                }
            }
        }
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        let first = match call.arguments.first().and_then(|arg| arg.as_expression()) {
            Some(Expression::StringLiteral(value)) => {
                Argument::Literal(crate::ast::js_literal(value).into_owned())
            }
            Some(Expression::Identifier(value)) => Argument::Reference(span(value.span)),
            _ => Argument::Other,
        };
        self.finite_calls.push(CallSite {
            callee: span(callee.span),
            first,
            count: call.arguments.len(),
            valid: !call.optional
                && call
                    .arguments
                    .iter()
                    .all(|arg| arg.as_expression().is_some()),
            context: self.current_function,
        });
    }
    pub(super) fn finite_function(&mut self, value: &AstFunction<'_>) {
        if !self.collect_finite
            || value.r#type != FunctionType::FunctionDeclaration
            || value.generator
            || value.params.rest.is_some()
            || value.params.items.iter().any(|param| {
                param.initializer.is_some()
                    || !matches!(param.pattern, BindingPattern::BindingIdentifier(_))
            })
        {
            return;
        }
        let (Some(id), Some(first)) = (&value.id, value.params.items.first()) else {
            return;
        };
        let BindingPattern::BindingIdentifier(parameter) = &first.pattern else {
            return;
        };
        self.finite_functions.push(Function {
            declaration: span(id.span),
            body: span(value.span),
            parameter: span(parameter.span),
        });
    }
    pub(super) fn finite_const(&mut self, value: &VariableDeclarator<'_>) {
        if !self.collect_finite {
            return;
        }
        let BindingPattern::BindingIdentifier(binding) = &value.id else {
            return;
        };
        let Some(init) = &value.init else {
            return;
        };
        if let Some(function) = self.current_function {
            if let Some(root) = finite_comments::root(span(binding.span), function, init) {
                self.finite_roots.push(root);
            }
        }
        let Expression::ArrowFunctionExpression(arrow) = init else {
            return;
        };
        self.finite_select(span(binding.span), arrow);
    }
    fn finite_select(
        &mut self,
        declaration: super::super::Span,
        arrow: &ArrowFunctionExpression<'_>,
    ) {
        let Some(enclosing) = self.current_function else {
            return;
        };
        if arrow.params.rest.is_some() || arrow.params.items.len() != 1 {
            return;
        }
        let first = &arrow.params.items[0];
        if first.initializer.is_some() {
            return;
        }
        let BindingPattern::BindingIdentifier(parameter) = &first.pattern else {
            return;
        };
        self.finite_selects.push(Select {
            declaration,
            body: span(arrow.span),
            enclosing,
            parameter: span(parameter.span),
        });
    }
}
