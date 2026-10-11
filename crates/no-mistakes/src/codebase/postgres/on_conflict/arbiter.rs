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
        SqlConflictArbiter::Expressions {
            columns,
            groups,
            unknown,
        } => {
            conflict.assignments.iter().find_map(|assignment| {
                if form_is_self(&assignment.form, &assignment.column) {
                    return None;
                }
                if columns.iter().any(|column| column.eq_ignore_ascii_case(&assignment.column)) {
                    if form_is_excluded(&assignment.form, &assignment.column) {
                        // A conflict on f(a,b) proves only that the complete
                        // expression matches. Replacing a alone can move the
                        // row's key, so every dependency of f must move too.
                        if groups.iter().any(|group| {
                            group.iter().any(|column| column.eq_ignore_ascii_case(&assignment.column))
                                && !group.iter().all(|column| {
                                    conflict.assignments.iter().any(|other| {
                                        other.column.eq_ignore_ascii_case(column)
                                            && form_is_excluded(&other.form, column)
                                    })
                                })
                        }) {
                            return Some(format!(
                                "ON CONFLICT expression arbiter depends on {}; assign EXCLUDED values to every column in its expression or leave it unchanged",
                                assignment.column
                            ));
                        }
                        return None;
                    }
                    return Some(format!(
                        "ON CONFLICT expression arbiter depends on {}; assign EXCLUDED.{} or leave it unchanged",
                        assignment.column, assignment.column
                    ));
                }
                if *unknown {
                    return Some(format!(
                        "ON CONFLICT expression arbiter cannot prove whether assignment to {} changes the conflict key",
                        assignment.column
                    ));
                }
                None
            })
        }
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
