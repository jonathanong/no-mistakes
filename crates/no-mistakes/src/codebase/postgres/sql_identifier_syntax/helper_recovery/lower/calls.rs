use super::{arguments, Expr, Scope};
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
        if member.property.name == "append" && call.arguments.len() == 1 {
            let Some(argument) = call.arguments[0].as_expression() else {
                return Expr::Opaque;
            };
            return Expr::Append(
                Box::new(scope.expression(&member.object)),
                Box::new(scope.expression(argument)),
                scope.mark(argument.span()),
            );
        }
        if member.property.name == "query" {
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
        let mut args = vec![scope.expression(&member.object)];
        args.extend(
            call.arguments
                .iter()
                .map(|arg| arguments::method_argument(scope, arg)),
        );
        return Expr::Call(format!("operation:method@{}", call.span.start), args);
    }
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return Expr::Opaque;
    };
    if scope.ctx.bindings.id(callee.span).is_none()
        && matches!(callee.name.as_str(), "Boolean" | "Number" | "String")
    {
        return Expr::Call(
            "operation:data".into(),
            call.arguments
                .iter()
                .map(|arg| arguments::method_argument(scope, arg))
                .collect(),
        );
    }
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
