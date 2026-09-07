use super::form_is_excluded;
use super::trigger::{fires_insert, fires_update};
use super::Catalog;
use crate::codebase::postgres::statement_facts::{
    SqlAssignmentFact, SqlOnConflictFact, SqlTriggerFact,
};
use crate::codebase::postgres::statements::form_is_stable;

pub(super) fn where_proves_noop(
    conflict: &SqlOnConflictFact,
    assigned: &[String],
    table: &str,
    catalog: &Catalog<'_>,
    insert_assignments: &[SqlAssignmentFact],
) -> bool {
    if conflict.where_proof.disjunctive || assigned.is_empty() {
        return false;
    }
    assigned.iter().any(|column| {
        let proven = conflict
            .where_proof
            .distinct_from_excluded
            .iter()
            .chain(conflict.where_proof.null_and_excluded_not_null.iter())
            .any(|name| name.eq_ignore_ascii_case(column));
        proven
            && insert_value_is_stable(insert_assignments, column)
            && !rewritten_by_applicable(catalog, table, assigned, column)
            && conflict.assignments.iter().any(|assignment| {
                assignment.column.eq_ignore_ascii_case(column)
                    && form_is_excluded(&assignment.form, column)
            })
    })
}

fn insert_value_is_stable(assignments: &[SqlAssignmentFact], column: &str) -> bool {
    assignments.iter().any(|assignment| {
        assignment.column.eq_ignore_ascii_case(column) && form_is_stable(&assignment.form)
    })
}

fn rewritten_by_applicable(
    catalog: &Catalog<'_>,
    table: &str,
    assigned: &[String],
    column: &str,
) -> bool {
    catalog.triggers.iter().any(|trigger| {
        trigger.table.eq_ignore_ascii_case(table)
            && mutates_proposed_row(trigger, assigned)
            && catalog.trigger_writes.iter().any(|(name, columns)| {
                name.eq_ignore_ascii_case(&trigger.function)
                    && columns
                        .iter()
                        .any(|written| written.eq_ignore_ascii_case(column))
            })
    })
}

fn mutates_proposed_row(trigger: &SqlTriggerFact, assigned: &[String]) -> bool {
    fires_update(trigger, assigned) || fires_insert(trigger)
}
