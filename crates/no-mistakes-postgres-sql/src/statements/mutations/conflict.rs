use super::SqlSelectFact;
use sqlparser::ast::{OnConflictAction, OnInsert};

pub(super) fn collect(
    sql: &str,
    on: Option<&OnInsert>,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    if let Some(OnInsert::OnConflict(conflict)) = on {
        if let OnConflictAction::DoUpdate(update) = &conflict.action {
            super::walk_side_queries(
                sql,
                &[],
                update.selection.as_ref(),
                ctes,
                selects,
                positions,
            );
            for assignment in &update.assignments {
                super::super::select::collect_predicate_shapes(&assignment.value, selects);
                super::factor::collect_exists_facts(sql, &assignment.value, positions, selects);
                super::super::select::walk_expr_at(
                    sql,
                    &assignment.value,
                    ctes,
                    false,
                    positions,
                    selects,
                );
            }
        }
    }
}
