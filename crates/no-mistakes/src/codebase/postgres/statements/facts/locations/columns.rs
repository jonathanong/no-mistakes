use super::{at, bare, Column, SqlColumnClause};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{
    BinaryOperator, Expr, OrderByExpr, Select, SelectItem, Spanned, TableWithJoins,
};

pub(super) fn selection(select: &Select, order: &[OrderByExpr]) -> Vec<Column> {
    let mut out = mutation(&select.from, select.selection.as_ref());
    for item in order {
        let resolved = if let Expr::Identifier(alias) =
            crate::codebase::postgres::idents::unwrap_expr(&item.expr)
        {
            select
                .projection
                .iter()
                .find_map(|item| match item {
                    SelectItem::ExprWithAlias { expr, alias: name }
                        if ident_key(alias) == ident_key(name) =>
                    {
                        Some(expr)
                    }
                    _ => None,
                })
                .unwrap_or(&item.expr)
        } else {
            &item.expr
        };
        record(
            resolved,
            SqlColumnClause::OrderBy,
            at(crate::codebase::postgres::idents::unwrap_expr(&item.expr).span()),
            &mut out,
        );
    }
    out
}
pub(super) fn mutation(tables: &[TableWithJoins], selection: Option<&Expr>) -> Vec<Column> {
    let mut out = Vec::new();
    if let Some(expr) = selection {
        walk(expr, SqlColumnClause::Where, &mut out);
    }
    for table in tables {
        for join in &table.joins {
            if let Some(expr) =
                crate::codebase::postgres::statements::select::join_expr(&join.join_operator)
            {
                walk(expr, SqlColumnClause::Join, &mut out);
            }
        }
    }
    out
}
fn walk(expr: &Expr, clause: SqlColumnClause, out: &mut Vec<Column>) {
    match expr {
        Expr::Nested(expr)
        | Expr::UnaryOp {
            op: sqlparser::ast::UnaryOperator::Not,
            expr,
        } => walk(expr, clause, out),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And | BinaryOperator::Or,
            right,
        } => {
            walk(left, clause, out);
            walk(right, clause, out);
        }
        Expr::BinaryOp {
            left,
            op:
                BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq,
            right,
        } => {
            record(
                left,
                clause,
                at(crate::codebase::postgres::idents::unwrap_expr(left).span()),
                out,
            );
            record(
                right,
                clause,
                at(crate::codebase::postgres::idents::unwrap_expr(right).span()),
                out,
            );
        }
        Expr::Between { expr, .. } | Expr::InList { expr, .. } | Expr::InSubquery { expr, .. } => {
            record(
                expr,
                clause,
                at(crate::codebase::postgres::idents::unwrap_expr(expr).span()),
                out,
            )
        }
        _ => {}
    }
}
fn record(expr: &Expr, clause: SqlColumnClause, at: (usize, usize), out: &mut Vec<Column>) {
    if let Some((qualifier, name)) = bare(expr) {
        out.push(Column {
            name,
            qualifier,
            clause,
            at,
        });
    }
}
