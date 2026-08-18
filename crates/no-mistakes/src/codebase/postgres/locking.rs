use super::parse::{parse_postgres_sql, PostgresParseError};
use super::schema::relation_name;
use sqlparser::ast::{
    BinaryOperator, Expr, Function, LockClause, LockType, NonBlock, Query, SetExpr, Statement,
    TableFactor, TableWithJoins,
};

/// Locking `SELECT` facts later lock-ordering rules can query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockingSelectMetadata {
    pub has_multi_row_predicate: bool,
    pub has_order_by: bool,
    pub skips_locked_rows: bool,
}

/// Parse `sql` and return one record per `SELECT` that uses `FOR UPDATE`.
pub fn extract_locking_select_metadata(
    sql: &str,
) -> Result<Vec<LockingSelectMetadata>, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    let mut locks = Vec::new();
    for statement in &statements {
        collect_from_statement(statement, &mut locks);
    }
    Ok(locks)
}

fn collect_from_statement(statement: &Statement, out: &mut Vec<LockingSelectMetadata>) {
    if let Statement::Query(query) = statement {
        collect_from_query(query, out);
    }
}

fn collect_from_query(query: &Query, out: &mut Vec<LockingSelectMetadata>) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_from_query(&cte.query, out);
        }
    }
    collect_from_set_expr(&query.body, out);
    if has_for_update(&query.locks) {
        out.push(LockingSelectMetadata {
            has_multi_row_predicate: set_expr_has_multi_row(&query.body),
            has_order_by: query.order_by.is_some(),
            skips_locked_rows: locks_skip_locked(&query.locks),
        });
    }
}

fn collect_from_set_expr(expr: &SetExpr, out: &mut Vec<LockingSelectMetadata>) {
    match expr {
        SetExpr::Select(select) => {
            if let Some(selection) = &select.selection {
                collect_queries_from_expr(selection, out);
            }
            for table in &select.from {
                collect_from_table_with_joins(table, out);
            }
        }
        SetExpr::Query(query) => collect_from_query(query, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_from_set_expr(left, out);
            collect_from_set_expr(right, out);
        }
        _ => {}
    }
}

fn collect_from_table_with_joins(table: &TableWithJoins, out: &mut Vec<LockingSelectMetadata>) {
    collect_from_table_factor(&table.relation, out);
    for join in &table.joins {
        collect_from_table_factor(&join.relation, out);
    }
}

fn collect_from_table_factor(factor: &TableFactor, out: &mut Vec<LockingSelectMetadata>) {
    if let TableFactor::Derived { subquery, .. } = factor {
        collect_from_query(subquery, out);
    }
}

fn collect_queries_from_expr(expr: &Expr, out: &mut Vec<LockingSelectMetadata>) {
    match unwrap_expr(expr) {
        Expr::Subquery(query)
        | Expr::InSubquery {
            subquery: query, ..
        } => {
            collect_from_query(query, out);
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_queries_from_expr(left, out);
            collect_queries_from_expr(right, out);
        }
        Expr::UnaryOp { expr, .. } => collect_queries_from_expr(expr, out),
        _ => {}
    }
}

fn has_for_update(locks: &[LockClause]) -> bool {
    locks.iter().any(|lock| lock.lock_type == LockType::Update)
}

fn locks_skip_locked(locks: &[LockClause]) -> bool {
    locks.iter().any(|lock| {
        lock.lock_type == LockType::Update && lock.nonblock == Some(NonBlock::SkipLocked)
    })
}

fn set_expr_has_multi_row(expr: &SetExpr) -> bool {
    match expr {
        SetExpr::Select(select) => select.selection.as_ref().is_some_and(expr_has_multi_row),
        SetExpr::Query(query) => set_expr_has_multi_row(&query.body),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_has_multi_row(left) || set_expr_has_multi_row(right)
        }
        _ => false,
    }
}

fn expr_has_multi_row(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::InList { .. } | Expr::InSubquery { .. } | Expr::InUnnest { .. } => true,
        Expr::AnyOp {
            compare_op: BinaryOperator::Eq,
            ..
        } => true,
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } if function_is_any(right) || function_is_any(left) => true,
        Expr::BinaryOp { left, right, .. } => expr_has_multi_row(left) || expr_has_multi_row(right),
        Expr::UnaryOp { expr, .. } => expr_has_multi_row(expr),
        _ => false,
    }
}

fn function_is_any(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Function(function) => function_name_is_any(function),
        _ => false,
    }
}

fn function_name_is_any(function: &Function) -> bool {
    relation_name(&function.name).eq_ignore_ascii_case("any")
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

#[cfg(test)]
mod tests;
