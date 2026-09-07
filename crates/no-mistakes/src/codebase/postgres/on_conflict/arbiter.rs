use super::{form_is_excluded, form_is_self};
use crate::codebase::postgres::statement_facts::{
    SqlConflictArbiter, SqlInsertFact, SqlOnConflictFact,
};

pub(super) fn judge(_insert: &SqlInsertFact, conflict: &SqlOnConflictFact) -> Option<String> {
    match &conflict.arbiter {
        SqlConflictArbiter::Columns(columns) => columns.iter().find_map(|column| {
            conflict.assignments.iter().find_map(|assignment| {
                if !assignment.column.eq_ignore_ascii_case(column) {
                    return None;
                }
                if form_is_excluded(&assignment.form, column)
                    || form_is_self(&assignment.form, column)
                {
                    return None;
                }
                Some(format!(
                    "ON CONFLICT arbiter column {} must be assigned EXCLUDED.{column} or left unchanged",
                    assignment.column
                ))
            })
        }),
        SqlConflictArbiter::Constraint(_) => conflict.assignments.iter().find_map(|assignment| {
            if form_is_excluded(&assignment.form, &assignment.column)
                || form_is_self(&assignment.form, &assignment.column)
            {
                None
            } else {
                Some(format!(
                    "named-constraint ON CONFLICT assignment to {} must be EXCLUDED or a self-reference",
                    assignment.column
                ))
            }
        }),
        SqlConflictArbiter::Unknown => conflict.assignments.iter().find_map(|assignment| {
            if form_is_self(&assignment.form, &assignment.column) {
                None
            } else {
                Some(format!(
                    "expression-index ON CONFLICT cannot prove arbiter membership for {}",
                    assignment.column
                ))
            }
        }),
    }
}
