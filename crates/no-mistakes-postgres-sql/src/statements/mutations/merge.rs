use super::SqlSelectFact;
use sqlparser::ast::{Merge, MergeAction, MergeInsertKind, MergeUpdateKind};

/// MERGE predicates and action subqueries share the existing shape/query walk.
pub(super) fn collect(
    sql: &str,
    statement: &Merge,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    super::walk_factor(sql, &statement.table, ctes, selects, positions);
    super::walk_factor(sql, &statement.source, ctes, selects, positions);
    super::walk_side_queries(sql, &[], Some(&statement.on), ctes, selects, positions);
    for clause in &statement.clauses {
        super::walk_side_queries(
            sql,
            &[],
            clause.predicate.as_ref(),
            ctes,
            selects,
            positions,
        );
        collect_action_shapes(sql, &clause.action, positions, selects);
        super::super::select::walk_node_at(sql, &clause.action, ctes, false, positions, selects);
    }
}

fn collect_action_shapes(
    sql: &str,
    action: &MergeAction,
    positions: super::super::value::PlaceholderPositions<'_>,
    selects: &mut Vec<SqlSelectFact>,
) {
    match action {
        MergeAction::Update(update) => {
            if let MergeUpdateKind::Set(assignments) = &update.kind {
                for assignment in assignments {
                    action_expr(sql, &assignment.value, positions, selects);
                }
            }
        }
        MergeAction::Insert(insert) => {
            if let MergeInsertKind::Values(values) = &insert.kind {
                for row in &values.rows {
                    for expr in &row.content {
                        action_expr(sql, expr, positions, selects);
                    }
                }
            }
        }
        MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
    }
}

fn action_expr(
    sql: &str,
    expr: &sqlparser::ast::Expr,
    positions: super::super::value::PlaceholderPositions<'_>,
    selects: &mut Vec<SqlSelectFact>,
) {
    super::super::select::collect_predicate_shapes(expr, selects);
    super::factor::collect_exists_facts(sql, expr, positions, selects);
}
