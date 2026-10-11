use super::parse::{parse_postgres_sql, PostgresParseError};
use super::schema::relation_name;
use super::{canonical_order_keys, CanonicalOrderKey};
use sqlparser::ast::{BinaryOperator, Expr, Function, LockClause, LockType, NonBlock, SetExpr};

mod collect;
mod relations;
mod single_row;
use collect::collect_from_statement;
pub(in crate::codebase::postgres) use collect::collect_located;
pub use single_row::JoinEquality;

/// Locking `SELECT` facts later lock-ordering rules can query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockingSelectMetadata {
    pub has_multi_row_predicate: bool,
    pub has_order_by: bool,
    pub skips_locked_rows: bool,
    pub tables: Option<Vec<String>>,
    pub table_qualifiers: Option<std::collections::BTreeMap<String, Vec<String>>>,
    pub order: Option<Vec<CanonicalOrderKey>>,
    /// Columns of each base relation in the FROM pinned to one bound value by a top-level
    /// `WHERE` or inner-join `ON` equality. With a catalog unique key they bound a relation
    /// to one row regardless of `IN` / `= ANY` filters on other columns.
    pub pinned_columns: Option<std::collections::BTreeMap<String, Vec<String>>>,
    /// Equalities between columns of two distinct base relations (top-level `AND` of `WHERE`
    /// or an inner join's `ON`): a relation pinned through one is as single-row as the other.
    pub join_equalities: Vec<JoinEquality>,
}

/// Parse `sql` and return one record per `SELECT` that uses `FOR UPDATE`.
///
/// No identifier is treated as a recovered interpolation; use
/// [`extract_locking_select_metadata_with_placeholders`] for embedded SQL.
pub fn extract_locking_select_metadata(
    sql: &str,
) -> Result<Vec<LockingSelectMetadata>, PostgresParseError> {
    extract_locking_select_metadata_with_placeholders(sql, &[])
}

/// Like [`extract_locking_select_metadata`], where `positions` are the SQL line and column of
/// each generated interpolation marker (`EmbeddedSqlCall::recovered_placeholder_positions`).
/// Only identifiers at those positions count as bound values, so user-authored text that
/// merely spells the marker stays a column.
pub fn extract_locking_select_metadata_with_placeholders(
    sql: &str,
    positions: &[(u32, u32)],
) -> Result<Vec<LockingSelectMetadata>, PostgresParseError> {
    let statements = parse_postgres_sql(sql)?;
    let mut locks = Vec::new();
    for statement in &statements {
        collect_from_statement(statement, &mut locks, positions);
    }
    Ok(locks)
}

fn order_keys(order: &sqlparser::ast::OrderBy) -> Option<Vec<CanonicalOrderKey>> {
    canonical_order_keys(order)
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
mod join_tests;
#[cfg(test)]
mod of_list_tests;
#[cfg(test)]
mod pinned_tests;
#[cfg(test)]
mod tests;
