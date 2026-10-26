use super::{Expr, Scope};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;
pub(super) fn lower(scope: &mut Scope<'_>, call: &CallExpression<'_>) -> Expr {
    if call
        .arguments
        .iter()
        .any(|arg| matches!(arg, Argument::SpreadElement(_)))
    {
        return Expr::Opaque;
    }
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
        if member.property.name == "append" && !member.optional {
            let [argument] = call.arguments.as_slice() else {
                return Expr::Opaque;
            };
            let Some(argument) = argument.as_expression() else {
                return Expr::Opaque;
            };
            return Expr::Append(
                Box::new(scope.expression(&member.object)),
                Box::new(scope.expression(argument)),
                scope.mark(argument.span()),
            );
        }
    }
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
        if member.property.name == "query" && !member.optional {
            if let Expression::Identifier(root) = unwrap_ts_wrappers(&member.object) {
                if let Some(id) = scope.ctx.bindings.id(root.span) {
                    scope.references.push((id, super::span(root.span)));
                    return Expr::Call(
                        format!("binding:{id}.query@{}", call.span.start),
                        call.arguments
                            .iter()
                            .filter_map(Argument::as_expression)
                            .map(|arg| scope.expression(arg))
                            .collect(),
                    );
                }
            }
        }
    }
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return Expr::Opaque;
    };
    let Some(id) = scope.ctx.bindings.id(callee.span) else {
        return Expr::Opaque;
    };
    Expr::Call(
        format!("binding:{id}@{}", call.span.start),
        call.arguments
            .iter()
            .filter_map(Argument::as_expression)
            .map(|arg| scope.expression(arg))
            .collect(),
    )
}
