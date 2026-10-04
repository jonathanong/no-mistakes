mod collector;
mod from;
mod nested;
mod shapes;
mod stars;
mod uses;
pub(super) use collector::collect_query_at;
pub(super) use nested::{walk_expr_at, walk_node_at};

pub(super) fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&sqlparser::ast::Expr> {
    from::join_expr(operator)
}

pub(super) fn collect_predicate_shapes(expr: &sqlparser::ast::Expr, out: &mut Vec<SqlSelectFact>) {
    let shapes = shapes::collect_predicate(expr);
    if shapes.not_in_subqueries.is_empty() && shapes.count_existence_checks.is_empty() {
        return;
    }
    out.push(SqlSelectFact {
        line: 1,
        tables: Vec::new(),
        predicate_sql: String::new(),
        exists_set_operations: Vec::new(),
        relations: Vec::new(),
        in_insert_select: false,
        not_in_subqueries: shapes.not_in_subqueries,
        not_in_columns: shapes.not_in_columns,
        count_existence_checks: shapes.count_existence_checks,
        star_projections: Vec::new(),
        column_uses: Vec::new(),
    });
}

use super::{SqlSelectFact, SqlStarProjectionFact};
use sqlparser::ast::Statement;

pub(super) fn mutation_column_uses(
    tables: &[sqlparser::ast::TableWithJoins],
    selection: Option<&sqlparser::ast::Expr>,
    ctes: &[String],
) -> Vec<super::SqlColumnUseFact> {
    uses::collect_mutation(tables, selection, ctes)
}

pub(super) fn returning_stars(sql: &str, statement: &Statement) -> Vec<SqlStarProjectionFact> {
    stars::returning(sql, statement)
}

pub(super) fn collect_with_placeholder_positions(
    sql: &str,
    statement: &Statement,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    match statement {
        Statement::Query(query) => collect_query_at(sql, query, &[], false, false, positions, out),
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query_at(sql, source, &[], true, false, positions, out);
            }
        }
        Statement::CreateView(view) => {
            collect_query_at(sql, &view.query, &[], false, false, positions, out)
        }
        Statement::CreateTable(table) => {
            if let Some(query) = table.query.as_deref() {
                collect_query_at(sql, query, &[], false, false, positions, out);
            }
        }
        Statement::Copy {
            source: sqlparser::ast::CopySource::Query(query),
            ..
        } => collect_query_at(sql, query, &[], false, false, positions, out),
        Statement::Explain { statement, .. } => {
            collect_with_placeholder_positions(sql, statement, positions, out)
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
