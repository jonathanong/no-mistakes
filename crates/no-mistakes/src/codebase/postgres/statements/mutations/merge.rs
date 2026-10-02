use super::SqlSelectFact;
use sqlparser::ast::{Merge, MergeAction, MergeInsertKind, MergeUpdateKind};

/// MERGE predicates and action subqueries share the existing shape/query walk.
pub(super) fn collect(
    sql: &str,
    statement: &Merge,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    super::walk_factor(sql, &statement.table, ctes, selects);
    super::walk_factor(sql, &statement.source, ctes, selects);
    super::walk_side_queries(sql, &[], Some(&statement.on), ctes, selects);
    for clause in &statement.clauses {
        super::walk_side_queries(sql, &[], clause.predicate.as_ref(), ctes, selects);
        collect_action_shapes(&clause.action, selects);
        super::super::select::walk_node(sql, &clause.action, ctes, false, selects);
    }
}

fn collect_action_shapes(action: &MergeAction, selects: &mut Vec<SqlSelectFact>) {
    match action {
        MergeAction::Update(update) => {
            if let MergeUpdateKind::Set(assignments) = &update.kind {
                for assignment in assignments {
                    super::super::select::collect_predicate_shapes(&assignment.value, selects);
                }
            }
        }
        MergeAction::Insert(insert) => {
            if let MergeInsertKind::Values(values) = &insert.kind {
                for row in &values.rows {
                    for expr in &row.content {
                        super::super::select::collect_predicate_shapes(expr, selects);
                    }
                }
            }
        }
        MergeAction::Delete { .. } | MergeAction::DoNothing { .. } => {}
    }
}
