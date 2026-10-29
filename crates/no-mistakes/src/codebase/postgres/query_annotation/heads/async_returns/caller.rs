//! A sole awaited factory result is consumed by the immediately next executor.
use super::Caller;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

pub(in super::super) fn callers(statements: &[Statement<'_>], body: oxc_span::Span) -> Vec<Caller> {
    statements
        .windows(2)
        .filter_map(|pair| {
            let Statement::VariableDeclaration(declaration) = &pair[0] else {
                return None;
            };
            if !declaration.kind.is_const() || declaration.declarations.len() != 1 {
                return None;
            }
            let binding = &declaration.declarations[0];
            let BindingPattern::BindingIdentifier(id) = &binding.id else {
                return None;
            };
            let Expression::AwaitExpression(awaited) = unwrap_ts_wrappers(binding.init.as_ref()?)
            else {
                return None;
            };
            let Expression::CallExpression(factory) = unwrap_ts_wrappers(&awaited.argument) else {
                return None;
            };
            if factory.optional
                || factory
                    .arguments
                    .iter()
                    .any(|arg| arg.as_expression().is_none())
            {
                return None;
            }
            let Expression::Identifier(callee) = unwrap_ts_wrappers(&factory.callee) else {
                return None;
            };
            let expression = match &pair[1] {
                Statement::ExpressionStatement(s) => &s.expression,
                Statement::VariableDeclaration(s) if s.declarations.len() == 1 => {
                    s.declarations[0].init.as_ref()?
                }
                _ => return None,
            };
            let expression = match unwrap_ts_wrappers(expression) {
                Expression::AwaitExpression(s) => unwrap_ts_wrappers(&s.argument),
                s => s,
            };
            let Expression::CallExpression(call) = expression else {
                return None;
            };
            if call.optional
                || call.arguments.len() != 1
                || !matches!(unwrap_ts_wrappers(&call.callee), Expression::Identifier(_))
            {
                return None;
            }
            let Expression::Identifier(reference) =
                unwrap_ts_wrappers(call.arguments[0].as_expression()?)
            else {
                return None;
            };
            if reference.name != id.name {
                return None;
            }
            Some(Caller {
                body: span(body),
                declaration: span(id.span),
                callee: span(callee.span),
                reference: span(reference.span),
                call: span(call.span()),
            })
        })
        .collect()
}
fn span(value: oxc_span::Span) -> super::super::Span {
    (value.start, value.end)
}
