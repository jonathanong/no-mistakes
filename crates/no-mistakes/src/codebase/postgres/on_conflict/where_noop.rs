use super::form_is_excluded;
use crate::codebase::postgres::statement_facts::SqlOnConflictFact;

pub(super) fn where_proves_noop(
    conflict: &SqlOnConflictFact,
    assigned: &[String],
    function: &str,
    writes: &[(String, Vec<String>)],
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
        let rewritten = writes.iter().any(|(name, columns)| {
            name.eq_ignore_ascii_case(function)
                && columns
                    .iter()
                    .any(|written| written.eq_ignore_ascii_case(column))
        });
        proven
            && !rewritten
            && conflict.assignments.iter().any(|assignment| {
                assignment.column.eq_ignore_ascii_case(column)
                    && form_is_excluded(&assignment.form, column)
            })
    })
}
