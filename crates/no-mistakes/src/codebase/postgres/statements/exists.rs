use super::SqlExistsSetOpFact;
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    BinaryOperator, Expr, GroupByExpr, JoinConstraint, JoinOperator, Query, Select, SelectItem,
    SetExpr, Spanned, Value, ValueWithSpan,
};
use sqlparser::tokenizer::Location;

pub(super) fn collect_from_select_at(
    sql: &str,
    select: &Select,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlExistsSetOpFact>,
) {
    collect_exists_at(sql, select.selection.as_ref(), positions, out);
    collect_exists_at(sql, select.having.as_ref(), positions, out);
    for item in &select.projection {
        match item {
            SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                collect_exists_at(sql, Some(expr), positions, out);
            }
            _ => {}
        }
    }
    if let GroupByExpr::Expressions(exprs, _) = &select.group_by {
        for expr in exprs {
            collect_exists_at(sql, Some(expr), positions, out);
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = join_on_expr(&join.join_operator) {
                collect_exists_at(sql, Some(expr), positions, out);
            }
        }
    }
}

pub(super) fn collect_exists_at(
    sql: &str,
    expr: Option<&Expr>,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlExistsSetOpFact>,
) {
    let Some(expr) = expr else {
        return;
    };
    match expr {
        Expr::Exists { subquery, .. } => {
            if set_expr_has_set_op(&subquery.body) {
                let (line, column) = exists_position(sql, subquery.span().start);
                out.push(SqlExistsSetOpFact {
                    restricted: set_expr_restricted_at(&subquery.body, positions),
                    correlated: super::exists_correlation::query_is_correlated(subquery),
                    line,
                    column,
                });
            }
            collect_query_exists_at(sql, subquery, positions, out);
        }
        Expr::Subquery(query) => collect_query_exists_at(sql, query, positions, out),
        other => crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
            collect_exists_at(sql, Some(child), positions, out);
        }),
    }
}

/// Locate the `EXISTS` keyword that opens the subquery starting at `start`,
/// so several `EXISTS` expressions each keep their own physical position.
fn exists_position(sql: &str, start: Location) -> (usize, usize) {
    let Some(line_index) = (start.line as usize).checked_sub(1) else {
        return first_exists_position(sql);
    };
    let line_offset: usize = sql
        .split_inclusive('\n')
        .take(line_index)
        .map(str::len)
        .sum();
    let line_text = sql[line_offset..].lines().next().unwrap_or("");
    let column_bytes = line_text
        .char_indices()
        .nth((start.column as usize).saturating_sub(1))
        .map_or(line_text.len(), |(index, _)| index);
    super::lines::last_word_position(sql, "exists", line_offset + column_bytes)
        .unwrap_or_else(|| first_exists_position(sql))
}

fn first_exists_position(sql: &str) -> (usize, usize) {
    let mut found = (1, 1);
    for (index, line) in sql.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if let Some(byte) = lower.find("exists") {
            found = (index + 1, lower[..byte].chars().count() + 1);
            break;
        }
    }
    found
}

fn collect_query_exists_at(
    sql: &str,
    query: &Query,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlExistsSetOpFact>,
) {
    collect_exists_from_set_at(sql, &query.body, positions, out);
}

fn collect_exists_from_set_at(
    sql: &str,
    expr: &SetExpr,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlExistsSetOpFact>,
) {
    match expr {
        SetExpr::Select(select) => collect_from_select_at(sql, select, positions, out),
        SetExpr::Query(query) => collect_query_exists_at(sql, query, positions, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_exists_from_set_at(sql, left, positions, out);
            collect_exists_from_set_at(sql, right, positions, out);
        }
        _ => {}
    }
}

fn join_on_expr(operator: &JoinOperator) -> Option<&Expr> {
    match operator {
        JoinOperator::Join(JoinConstraint::On(expr))
        | JoinOperator::Inner(JoinConstraint::On(expr))
        | JoinOperator::Left(JoinConstraint::On(expr))
        | JoinOperator::LeftOuter(JoinConstraint::On(expr))
        | JoinOperator::Right(JoinConstraint::On(expr))
        | JoinOperator::RightOuter(JoinConstraint::On(expr))
        | JoinOperator::FullOuter(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}

fn set_expr_has_set_op(expr: &SetExpr) -> bool {
    matches!(expr, SetExpr::SetOperation { .. })
        || matches!(expr, SetExpr::Query(query) if set_expr_has_set_op(&query.body))
}

fn set_expr_restricted_at(
    expr: &SetExpr,
    positions: super::value::PlaceholderPositions<'_>,
) -> bool {
    match expr {
        SetExpr::Select(select) => select
            .selection
            .as_ref()
            .is_some_and(|expr| expr_is_restricted_at(expr, positions)),
        SetExpr::Query(query) => set_expr_restricted_at(&query.body, positions),
        SetExpr::SetOperation { left, right, .. } => {
            set_expr_restricted_at(left, positions) && set_expr_restricted_at(right, positions)
        }
        _ => false,
    }
}

fn expr_is_restricted_at(expr: &Expr, positions: super::value::PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => expr_is_restricted_at(left, positions) || expr_is_restricted_at(right, positions),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => expr_is_restricted_at(left, positions) && expr_is_restricted_at(right, positions),
        Expr::BinaryOp { left, right, .. } => column_bound_to_const_at(left, right, positions),
        _ => false,
    }
}

fn column_bound_to_const_at(
    left: &Expr,
    right: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> bool {
    (is_relation_column_at(left, positions) && is_const_or_placeholder_at(right, positions))
        || (is_relation_column_at(right, positions) && is_const_or_placeholder_at(left, positions))
}

fn is_relation_column_at(expr: &Expr, positions: super::value::PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => !super::value::is_placeholder_ident_at(ident, positions),
        Expr::CompoundIdentifier(_) => true,
        _ => false,
    }
}

fn is_const_or_placeholder_at(
    expr: &Expr,
    positions: super::value::PlaceholderPositions<'_>,
) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(ValueWithSpan { value, .. }) => matches!(
            value,
            Value::Placeholder(_)
                | Value::Number(_, _)
                | Value::SingleQuotedString(_)
                | Value::Boolean(_)
        ),
        Expr::Identifier(ident) => super::value::is_placeholder_ident_at(ident, positions),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
