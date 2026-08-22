use super::parse::{parse_postgres_sql, PostgresParseError};
use sqlparser::ast::{
    Expr, Insert, LimitClause, Query, SetExpr, Statement, TableFactor, TableWithJoins,
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
        SetExpr::Select(select) => {
            select.selection.as_ref().is_some_and(expr_has_offset_query)
                || select.from.iter().any(table_with_joins_has_offset)
        }
        SetExpr::Query(query) => query_has_offset(query),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_has_offset(left) || set_expr_has_offset(right)
        }
        _ => false,
    }
}

fn table_with_joins_has_offset(table: &TableWithJoins) -> bool {
    table_factor_has_offset(&table.relation)
        || table
            .joins
            .iter()
            .any(|join| table_factor_has_offset(&join.relation))
}

fn table_factor_has_offset(factor: &TableFactor) -> bool {
    matches!(factor, TableFactor::Derived { subquery, .. } if query_has_offset(subquery))
}

fn expr_has_offset_query(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Subquery(query)
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
