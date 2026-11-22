use super::helpers::{generated_name, qualifier};
use super::PlaceholderPositions;
use crate::codebase::postgres::idents::{ident_key, object_name_identity, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator, Expr, FromTable, LimitClause, SelectItem, SetExpr, Statement, TableFactor,
    TableWithJoins,
};

pub(super) fn linked(stmt: &Statement, positions: PlaceholderPositions<'_>) -> bool {
    let Statement::Query(query) = stmt else {
        return false;
    };
    let Some(with) = &query.with else {
        return false;
    };
    let ([cte], SetExpr::Delete(delete)) = (with.cte_tables.as_slice(), query.body.as_ref()) else {
        return false;
    };
    let Statement::Delete(delete) = delete else {
        return false;
    };
    let SetExpr::Select(select) = cte.query.body.as_ref() else {
        return false;
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
        return false;
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
        return false;
    }
    let [SelectItem::UnnamedExpr(column)] = select.projection.as_slice() else {
        return false;
    };
    let column = match unwrap_expr(column) {
        Expr::Identifier(column) => ident_key(column),
        Expr::CompoundIdentifier(parts) if parts.len() == 2 && ident_key(&parts[0]) == locked.1 => {
            ident_key(&parts[1])
        }
        _ => return false,
    };
    let Some(Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    }) = delete.selection.as_ref().map(unwrap_expr)
    else {
        return false;
    };
    let matches = |expr: &Expr, table: &str| {
        matches!(unwrap_expr(expr), Expr::CompoundIdentifier(parts)
            if parts.len() == 2 && ident_key(&parts[0]) == table && ident_key(&parts[1]) == column)
    };
    (matches(left, &target.1) && matches(right, &using.1))
        || (matches(right, &target.1) && matches(left, &using.1))
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
