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
    pub position: u8,
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
    pub position: u8,
    pub shape: CallShape,
}

pub(super) fn function(value: &Function<'_>) -> [Option<FunctionSyntax>; 2] {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return [None, None];
    }
    let Some(body) = value.body.as_ref() else {
        return [None, None];
    };
    let params = &value.params;
    if params.rest.is_some() || params.items.is_empty() {
        return [None, None];
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
            return [None, None];
        }
    }
    std::array::from_fn(|position| {
        // Keep the existing first-parameter grammar; the new position has no defaults.
        if position == 1
            && !(params.items.len() == 2
                && params.items.iter().all(|param| {
                    param.initializer.is_none()
                        && matches!(param.pattern, BindingPattern::BindingIdentifier(_))
                }))
        {
            return None;
        }
        let BindingPattern::BindingIdentifier(parameter) = &params.items.get(position)?.pattern
        else {
            return None;
        };
        Some(FunctionSyntax {
            parameter: (parameter.span.start, parameter.span.end),
            position: position as u8,
            function: (value.span.start, value.span.end),
            body: (body.span.start, body.span.end),
            signature,
        })
    })
}

pub(super) fn call(expression: &Expression<'_>) -> [Option<CallSyntax>; 2] {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
        return [None, None];
    };
    if call.optional
        || call
            .arguments
            .iter()
            .any(|arg| arg.as_expression().is_none())
    {
        return [None, None];
    }
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return [None, None];
    };
    let shape =
        CallShape {
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
        };
    std::array::from_fn(|position| {
        if position == 1 && call.arguments.len() != 2 {
            return None;
        }
        let Expression::Identifier(builder) =
            unwrap_ts_wrappers(call.arguments.get(position)?.as_expression()?)
        else {
            return None;
        };
        Some(CallSyntax {
            span: (call.span.start, call.span.end),
            callee: (callee.span.start, callee.span.end),
            builder: (builder.span.start, builder.span.end),
            position: position as u8,
            shape,
        })
    })
}
