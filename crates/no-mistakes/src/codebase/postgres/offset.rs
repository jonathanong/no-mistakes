use super::parse::{parse_postgres_sql, PostgresParseError};
use sqlparser::ast::{
    CreateTable, CreateView, Delete, Expr, Insert, JoinConstraint, JoinOperator, LimitClause,
    Query, Select, SelectItem, SetExpr, Statement, TableFactor, TableWithJoins, Update, Values,
};

/// Parse `sql` and report whether any query uses an `OFFSET` clause.
pub fn sql_has_offset_clause(sql: &str) -> Result<bool, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    Ok(statements.iter().any(statement_has_offset))
}

fn statement_has_offset(statement: &Statement) -> bool {
    match statement {
        Statement::Query(query) => query_has_offset(query),
        Statement::Insert(Insert { source, .. }) => {
            source.as_ref().is_some_and(|query| query_has_offset(query))
        }
        Statement::Update(Update {
            assignments,
            selection,
            table,
            from,
            ..
        }) => {
            assignments
                .iter()
                .any(|assignment| expr_has_offset_query(&assignment.value))
                || selection.as_ref().is_some_and(expr_has_offset_query)
                || table_with_joins_has_offset(table)
                || matches!(
                    from.as_ref(),
                    Some(sqlparser::ast::UpdateTableFromKind::AfterSet(tables))
                        if tables.iter().any(table_with_joins_has_offset)
                )
        }
        Statement::Delete(Delete {
            selection, using, ..
        }) => {
            selection.as_ref().is_some_and(expr_has_offset_query)
                || using
                    .as_ref()
                    .is_some_and(|tables| tables.iter().any(table_with_joins_has_offset))
        }
        Statement::CreateTable(CreateTable { query, .. }) => {
            query.as_deref().is_some_and(query_has_offset)
        }
        Statement::CreateView(CreateView { query, .. }) => query_has_offset(query),
        Statement::Explain { statement, .. } => statement_has_offset(statement),
        _ => false,
    }
}

fn query_has_offset(query: &Query) -> bool {
    if limit_clause_has_offset(query.limit_clause.as_ref()) {
        return true;
    }
    if query.with.as_ref().is_some_and(|with| {
        with.cte_tables
            .iter()
            .any(|cte| query_has_offset(&cte.query))
    }) {
        return true;
    }
    if let Some(order_by) = &query.order_by {
        if let sqlparser::ast::OrderByKind::Expressions(exprs) = &order_by.kind {
            if exprs.iter().any(|item| expr_has_offset_query(&item.expr)) {
                return true;
            }
        }
    }
    set_expr_has_offset(&query.body)
}

fn limit_clause_has_offset(clause: Option<&LimitClause>) -> bool {
    matches!(
        clause,
        Some(LimitClause::LimitOffset {
            offset: Some(_),
            ..
        }) | Some(LimitClause::OffsetCommaLimit { .. })
    )
}

fn set_expr_has_offset(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(select) => select_has_offset(select),
        SetExpr::Query(query) => query_has_offset(query),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_has_offset(left) || set_expr_has_offset(right)
        }
        SetExpr::Values(Values { rows, .. }) => {
            rows.iter().any(|row| row.iter().any(expr_has_offset_query))
        }
        _ => false,
    }
}

fn select_has_offset(select: &Select) -> bool {
    select.projection.iter().any(select_item_has_offset)
        || select.selection.as_ref().is_some_and(expr_has_offset_query)
        || select.having.as_ref().is_some_and(expr_has_offset_query)
        || select.from.iter().any(table_with_joins_has_offset)
}

fn select_item_has_offset(item: &SelectItem) -> bool {
    match item {
        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
            expr_has_offset_query(expr)
        }
        _ => false,
    }
}

fn table_with_joins_has_offset(table: &TableWithJoins) -> bool {
    table_factor_has_offset(&table.relation)
        || table.joins.iter().any(|join| {
            table_factor_has_offset(&join.relation) || join_operator_has_offset(&join.join_operator)
        })
}

fn table_factor_has_offset(factor: &TableFactor) -> bool {
    matches!(factor, TableFactor::Derived { subquery, .. } if query_has_offset(subquery))
}

fn join_operator_has_offset(op: &JoinOperator) -> bool {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c) => constraint_has_offset(c),
        _ => false,
    }
}

fn constraint_has_offset(constraint: &JoinConstraint) -> bool {
    matches!(constraint, JoinConstraint::On(expr) if expr_has_offset_query(expr))
}

fn expr_has_offset_query(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Subquery(query)
        | Expr::Exists {
            subquery: query, ..
        }
        | Expr::InSubquery {
            subquery: query, ..
        } => query_has_offset(query),
        Expr::BinaryOp { left, right, .. } => {
            expr_has_offset_query(left) || expr_has_offset_query(right)
        }
        Expr::UnaryOp { expr, .. } => expr_has_offset_query(expr),
        _ => false,
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

#[cfg(test)]
mod tests;
