use crate::codebase::postgres::{SqlConflictInsertFact, SqlStatementFileFacts};

pub(super) struct PreparedConflicts<'a> {
    pub conflicts: &'a [SqlConflictInsertFact],
    /// Report `unanalyzable-sql` when fail-closed. Complete sibling conflicts stay in `conflicts`.
    pub unanalyzable: bool,
}

pub(super) fn prepared_conflicts(statement: &SqlStatementFileFacts) -> PreparedConflicts<'_> {
    let Some(locations) = statement.variant_locations.as_ref() else {
        return PreparedConflicts {
            conflicts: &[],
            unanalyzable: true,
        };
    };
    if !statement.parse_failed {
        let unanalyzable = locations.conflict_error.is_some();
        return PreparedConflicts {
            conflicts: if unanalyzable {
                &[]
            } else {
                &locations.conflicts
            },
            unanalyzable,
        };
    }
    PreparedConflicts {
        conflicts: &locations.reliable_conflicts,
        unanalyzable: locations.conflict_failure_overlaps,
    }
}
