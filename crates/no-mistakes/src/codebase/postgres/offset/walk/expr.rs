use super::super::OffsetUse;
use super::query_offsets;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, JoinConstraint, JoinOperator,
    TableFactor, TableFunctionArgs, TableWithJoins,
};

pub(super) fn table_with_joins_offsets(table: &TableWithJoins, out: &mut Vec<OffsetUse>) {
    table_factor_offsets(&table.relation, out);
    for join in &table.joins {
        table_factor_offsets(&join.relation, out);
        join_operator_offsets(&join.join_operator, out);
    }
}

fn table_factor_offsets(factor: &TableFactor, out: &mut Vec<OffsetUse>) {
    match factor {
        TableFactor::Derived { subquery, .. } => query_offsets(subquery, out),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => table_with_joins_offsets(table_with_joins, out),
        TableFactor::Table {
            args: Some(TableFunctionArgs { args, .. }),
            ..
        } => {
            for arg in args {
                if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                    expr_offsets(expr, out);
                }
            }
        }
        _ => {}
    }
}

fn join_operator_offsets(op: &JoinOperator, out: &mut Vec<OffsetUse>) {
    let constraint = match op {
        JoinOperator::Join(constraint)
        | JoinOperator::Inner(constraint)
        | JoinOperator::Left(constraint)
        | JoinOperator::LeftOuter(constraint)
        | JoinOperator::Right(constraint)
        | JoinOperator::RightOuter(constraint)
        | JoinOperator::FullOuter(constraint) => constraint,
        _ => return,
    };
    if let JoinConstraint::On(expr) = constraint {
        expr_offsets(expr, out);
    }
}

pub(super) fn expr_offsets(expr: &Expr, out: &mut Vec<OffsetUse>) {
    match peel(expr) {
        Expr::Subquery(query)
        | Expr::Exists {
            subquery: query, ..
        }
        | Expr::InSubquery {
            subquery: query, ..
        } => query_offsets(query, out),
        Expr::BinaryOp { left, right, .. } => {
            expr_offsets(left, out);
            expr_offsets(right, out);
        }
        Expr::UnaryOp { expr, .. } => expr_offsets(expr, out),
        Expr::Function(Function { args, .. }) => function_args_offsets(args, out),
        _ => {}
    }
}

fn function_args_offsets(args: &FunctionArguments, out: &mut Vec<OffsetUse>) {
    let FunctionArguments::List(list) = args else {
        return;
    };
    for arg in &list.args {
        if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
            expr_offsets(expr, out);
        }
    }
}

fn peel(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => peel(inner),
        other => other,
    }
}
