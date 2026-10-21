use crate::codebase::postgres::statements::SqlVariantLocations;
use crate::codebase::postgres::SqlConflictInsertFact;

pub(super) struct PreparedConflicts<'a> {
    pub conflicts: &'a [SqlConflictInsertFact],
    /// Report `unanalyzable-sql` when fail-closed. Complete sibling conflicts stay in `conflicts`.
    pub unanalyzable: bool,
}

pub(super) fn prepared_conflicts(
    locations: &SqlVariantLocations,
    parse_failed: bool,
) -> PreparedConflicts<'_> {
    if !parse_failed {
        // `conflicts` stays empty when analysis returns Err, so the error arm
        // does not need a second empty slice.
        return PreparedConflicts {
            conflicts: &locations.conflicts,
            unanalyzable: locations.conflict_error.is_some(),
        };
    }
    PreparedConflicts {
        conflicts: &locations.reliable_conflicts,
        unanalyzable: locations.conflict_failure_overlaps,
    }
}

#[cfg(test)]
mod tests;
