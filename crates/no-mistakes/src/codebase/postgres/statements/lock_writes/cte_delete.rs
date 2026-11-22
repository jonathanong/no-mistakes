use super::helpers::{generated_name, qualifier};
use super::PlaceholderPositions;
use crate::codebase::postgres::idents::{ident_key, object_name_identity, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator, Expr, FromTable, LimitClause, SelectItem, SetExpr, Statement, TableFactor,
    TableWithJoins,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CteWriteLink {
    Complete,
    RequiresNonNull(String),
}

pub(super) fn linked(
    stmt: &Statement,
    positions: PlaceholderPositions<'_>,
) -> Option<CteWriteLink> {
    let Statement::Query(query) = stmt else {
        return None;
    };
    let Some(with) = &query.with else {
        return None;
    };
    let ([cte], SetExpr::Delete(delete)) = (with.cte_tables.as_slice(), query.body.as_ref()) else {
        return None;
    };
    let Statement::Delete(delete) = delete else {
        return None;
    };
    let SetExpr::Select(select) = cte.query.body.as_ref() else {
        return None;
    };
    let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) = &delete.from;
    let (Some(target), Some(using), Some(locked)) = (
        simple_table(from, positions),
        delete
            .using
            .as_deref()
            .and_then(|tables| simple_table(tables, positions)),
        simple_table(&select.from, positions),
    ) else {
        return None;
    };
    if with.recursive
        || cte.query.with.is_some()
        || cte.query.locks.is_empty()
        || !cte.query.limit_clause.as_ref().is_none_or(|limit| {
            matches!(limit, LimitClause::LimitOffset { offset: None, limit_by, .. } if limit_by.is_empty())
        })
        || !cte.alias.columns.is_empty()
        || delete.limit.is_some()
        || target.0 != locked.0
        || using.0 != ident_key(&cte.alias.name)
        || using.1 == target.1
    {
        return None;
    }
    let [SelectItem::UnnamedExpr(column)] = select.projection.as_slice() else {
        return None;
    };
    let column = match unwrap_expr(column) {
        Expr::Identifier(column) => ident_key(column),
        Expr::CompoundIdentifier(parts) if parts.len() == 2 && ident_key(&parts[0]) == locked.1 => {
            ident_key(&parts[1])
        }
        _ => return None,
    };
    let (left, right, null_safe) = match unwrap_expr(delete.selection.as_ref()?) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => (left, right, false),
        Expr::IsNotDistinctFrom(left, right) => (left, right, true),
        _ => return None,
    };
    let matches = |expr: &Expr, table: &str| {
        matches!(unwrap_expr(expr), Expr::CompoundIdentifier(parts)
            if parts.len() == 2 && ident_key(&parts[0]) == table && ident_key(&parts[1]) == column)
    };
    if !(matches(left, &target.1) && matches(right, &using.1)
        || matches(right, &target.1) && matches(left, &using.1))
    {
        return None;
    }
    if null_safe
        || select
            .selection
            .as_ref()
            .is_some_and(|expr| non_null(expr, &locked.1, &column))
    {
        Some(CteWriteLink::Complete)
    } else {
        Some(CteWriteLink::RequiresNonNull(column))
    }
}

fn non_null(expr: &Expr, table: &str, column: &str) -> bool {
    let projected = |expr: &Expr| match unwrap_expr(expr) {
        Expr::Identifier(ident) => ident_key(ident) == column,
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
            ident_key(&parts[0]) == table && ident_key(&parts[1]) == column
        }
        _ => false,
    };
    match unwrap_expr(expr) {
        Expr::IsNotNull(value) => projected(value),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => non_null(left, table, column) || non_null(right, table, column),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            (projected(left) && matches!(unwrap_expr(right), Expr::Value(_)))
                || (projected(right) && matches!(unwrap_expr(left), Expr::Value(_)))
        }
        _ => false,
    }
}

fn simple_table(
    tables: &[TableWithJoins],
    positions: PlaceholderPositions<'_>,
) -> Option<(String, String)> {
    let [table] = tables else { return None };
    let TableFactor::Table {
        name,
        alias,
        args: None,
        inheritance: None,
        sample: None,
        ..
    } = &table.relation
    else {
        return None;
    };
    if !table.joins.is_empty()
        || generated_name(name, positions)
        || alias
            .as_ref()
            .is_some_and(|alias| !alias.columns.is_empty())
    {
        return None;
    }
    Some((object_name_identity(name), qualifier(&table.relation)?))
}
