//! Prefix-preserving effects of a discarded call on a trusted local builder.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, Function, FunctionType};

mod defaults;
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
    pub signature: Signature,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Signature {
    required_objects: u16,
    empty_defaults: u16,
}

impl Signature {
    pub(crate) fn accepts(self, shape: CallShape) -> bool {
        (0..16)
            .filter(|index| self.required_objects & (1 << index) != 0)
            .all(|index| {
                if index < shape.arg_count {
                    shape.closed_objects & (1 << index) != 0
                } else {
                    self.empty_defaults & (1 << index) != 0
                }
            })
    }

    pub(super) fn plain(self) -> bool {
        self.required_objects == 0
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CallShape {
    arg_count: usize,
    closed_objects: u16,
}

#[derive(Clone, Copy)]
pub(super) struct CallSyntax {
    pub span: Span,
    pub callee: Span,
    pub builder: Span,
    pub shape: CallShape,
}

pub(super) fn function(value: &Function<'_>) -> Option<FunctionSyntax> {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return None;
    }
    let body = value.body.as_ref()?;
    let params = &value.params;
    if params.rest.is_some() || params.items.is_empty() {
        return None;
    }
    let mut signature = Signature::default();
    for (index, param) in params.items.iter().enumerate() {
        if !param.decorators.is_empty()
            || !defaults::plain_parameter(
                index,
                &param.pattern,
                param.initializer.as_deref(),
                &mut signature,
            )
        {
            return None;
        }
    }
    let BindingPattern::BindingIdentifier(parameter) = &params.items[0].pattern else {
        return None;
    };
    Some(FunctionSyntax {
        parameter: (parameter.span.start, parameter.span.end),
        function: (value.span.start, value.span.end),
        body: (body.span.start, body.span.end),
        signature,
    })
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
        shape: CallShape {
            arg_count: call.arguments.len(),
            closed_objects: call.arguments.iter().take(16).enumerate().fold(
                0,
                |bits, (index, arg)| {
                    if arg.as_expression().is_some_and(defaults::closed_object) {
                        bits | (1 << index)
                    } else {
                        bits
                    }
                },
            ),
        },
    })
}
