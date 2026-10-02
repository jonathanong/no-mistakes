use super::SqlSelectFact;
use sqlparser::ast::{OnConflictAction, OnInsert};

pub(super) fn collect(
    sql: &str,
    on: Option<&OnInsert>,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    if let Some(OnInsert::OnConflict(conflict)) = on {
        if let OnConflictAction::DoUpdate(update) = &conflict.action {
            super::walk_side_queries(sql, &[], update.selection.as_ref(), ctes, selects);
            for assignment in &update.assignments {
                super::super::select::collect_predicate_shapes(&assignment.value, selects);
                super::super::select::walk_expr(sql, &assignment.value, ctes, false, selects);
            }
        }
    }
}
