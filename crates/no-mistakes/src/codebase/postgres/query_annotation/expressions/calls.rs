use super::{expression, Expr};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn calls(value: &CallExpression<'_>, source: &str) -> Expr {
    let args = value
        .arguments
        .iter()
        .map(|arg| {
            if let oxc_ast::ast::Argument::SpreadElement(spread) = arg {
                Expr::Spread(Box::new(expression(&spread.argument, source)))
            } else {
                expression(
                    arg.as_expression()
                        .expect("non-spread arguments are expressions"),
                    source,
                )
            }
        })
        .collect::<Vec<_>>();
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&value.callee) {
        if member.property.name == "append" && args.len() == 1 {
            return Expr::Append(
                Box::new(expression(&member.object, source)),
                Box::new(args[0].clone()),
            );
        }
    }
    Expr::Call {
        // Optional calls are handled conservatively by the ChainExpression arm.
        callee: Box::new(expression(&value.callee, source)),
        args,
        start: value.span.start,
    }
}
