use crate::codebase::postgres::parse_postgres_expression;
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments};
use std::collections::BTreeMap;

pub(super) fn substitute_target_columns(
    expression: &str,
    projections: &BTreeMap<String, String>,
) -> Option<String> {
    let mut expression = parse_postgres_expression(expression)?;
    substitute_expression(&mut expression, projections)?;
    Some(expression.to_string())
}

fn substitute_expression(
    expression: &mut Expr,
    projections: &BTreeMap<String, String>,
) -> Option<()> {
    match expression {
        Expr::Identifier(identifier) => {
            *expression = parse_postgres_expression(
                projections.get(&identifier.value.to_ascii_lowercase())?,
            )?;
        }
        Expr::Nested(inner)
        | Expr::UnaryOp { expr: inner, .. }
        | Expr::Cast { expr: inner, .. } => {
            substitute_expression(inner, projections)?;
        }
        Expr::BinaryOp { left, right, .. }
        | Expr::IsDistinctFrom(left, right)
        | Expr::IsNotDistinctFrom(left, right) => {
            substitute_expression(left, projections)?;
            substitute_expression(right, projections)?;
        }
        Expr::Function(function) => substitute_function_arguments(&mut function.args, projections)?,
        Expr::Value(_) | Expr::TypedString { .. } => {}
        _ => return None,
    }
    Some(())
}

fn substitute_function_arguments(
    arguments: &mut FunctionArguments,
    projections: &BTreeMap<String, String>,
) -> Option<()> {
    let FunctionArguments::List(arguments) = arguments else {
        return None;
    };
    for argument in &mut arguments.args {
        let (FunctionArg::Unnamed(FunctionArgExpr::Expr(expression))
        | FunctionArg::Named {
            arg: FunctionArgExpr::Expr(expression),
            ..
        }) = argument
        else {
            return None;
        };
        substitute_expression(expression, projections)?;
    }
    Some(())
}
