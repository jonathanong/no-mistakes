//! Prefix-preserving effects of a discarded call on a trusted local builder.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, Function, FunctionType};

mod prepare;
mod summary;
pub(crate) use prepare::Pending;
pub(super) use prepare::{prepare, Prepared};

#[derive(Clone, Debug)]
pub(crate) struct Conditional {
    pub span: Span,
    pub required: Vec<Span>,
}

#[derive(Clone, Copy)]
pub(super) struct FunctionSyntax {
    pub parameter: Span,
    pub function: Span,
    pub body: Span,
}

#[derive(Clone, Copy)]
pub(super) struct CallSyntax {
    pub span: Span,
    pub callee: Span,
    pub builder: Span,
}

pub(super) fn function(value: &Function<'_>) -> Option<FunctionSyntax> {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return None;
    }
    let body = value.body.as_ref()?;
    let params = &value.params;
    if params.rest.is_some()
        || params.items.is_empty()
        || params.items.iter().enumerate().any(|(index, param)| {
            param.initializer.is_some()
                || !param.decorators.is_empty()
                || !plain_parameter(index, &param.pattern)
        })
    {
        return None;
    }
    let BindingPattern::BindingIdentifier(parameter) = &params.items[0].pattern else {
        return None;
    };
    Some(FunctionSyntax {
        parameter: (parameter.span.start, parameter.span.end),
        function: (value.span.start, value.span.end),
        body: (body.span.start, body.span.end),
    })
}

fn plain_parameter(index: usize, pattern: &BindingPattern<'_>) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(_) => true,
        BindingPattern::ObjectPattern(object) if index > 0 && object.rest.is_none() => {
            object.properties.iter().all(|property| {
                !property.computed
                    && crate::ast::property_key_name(&property.key)
                        .is_some_and(|name| name != "__proto__")
                    && matches!(property.value, BindingPattern::BindingIdentifier(_))
            })
        }
        _ => false,
    }
}

pub(super) fn call(expression: &Expression<'_>) -> Option<CallSyntax> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    if call.optional
        || call
            .arguments
            .iter()
            .any(|arg| arg.as_expression().is_none())
    {
        return None;
    }
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let Expression::Identifier(builder) =
        unwrap_ts_wrappers(call.arguments.first()?.as_expression()?)
    else {
        return None;
    };
    Some(CallSyntax {
        span: (call.span.start, call.span.end),
        callee: (callee.span.start, callee.span.end),
        builder: (builder.span.start, builder.span.end),
    })
}
