use super::{defaults, CallShape, CallSyntax};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(in super::super) fn call(expression: &Expression<'_>) -> [Option<CallSyntax>; 2] {
    let original = expression;
    let mut expression = expression;
    let mut returned_append = false;
    while let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) {
        let Some(receiver) = super::super::append::prefix_receiver(call) else {
            break;
        };
        returned_append = true;
        expression = receiver;
    }
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
            returned_append,
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
            span: {
                use oxc_span::GetSpan;
                let span = original.span();
                (span.start, span.end)
            },
            callee: (callee.span.start, callee.span.end),
            builder: (builder.span.start, builder.span.end),
            position: position as u8,
            shape,
        })
    })
}
