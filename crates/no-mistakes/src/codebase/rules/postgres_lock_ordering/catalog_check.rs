//! Schema-catalog proofs for one locking `SELECT`.

use crate::codebase::postgres::{LockingSelectMetadata, SchemaCatalog};

/// Generated interpolation marker the embedded-SQL collector substitutes for `${...}`.
const INTERPOLATION_MARKER: &str = "sql_placeholder_";

/// Whether the lock touches at most one row of every locked table: some catalog unique
/// key of each table has all of its columns pinned by top-level `WHERE` equalities, so an
/// `IN` or `= ANY` elsewhere can only narrow that row, never widen it.
pub(super) fn locks_single_row(lock: &LockingSelectMetadata, catalog: &SchemaCatalog) -> bool {
    let (Some(tables), Some(pinned)) = (&lock.tables, &lock.pinned_columns) else {
        return false;
    };
    !tables.is_empty()
        && tables.iter().all(|table| {
            pinned.get(table).is_some_and(|pinned| {
                catalog
                    .unique_keys(table)
                    .iter()
                    .any(|key| !key.is_empty() && key.iter().all(|column| pinned.contains(column)))
            })
        })
}

/// Whether every locked relation's catalog key prefix starts the `ORDER BY`.
pub(super) fn orders_by_catalog_key(lock: &LockingSelectMetadata, catalog: &SchemaCatalog) -> bool {
    lock.tables
        .as_deref()
        .zip(lock.order.as_deref())
        .is_some_and(|(tables, order)| {
            !tables.is_empty()
                && tables.iter().all(|table| {
                    lock.table_qualifiers
                        .as_ref()
                        .and_then(|qualifiers| qualifiers.get(table))
                        .is_some_and(|qualifiers| {
                            catalog.has_canonical_prefix_for_qualifiers(table, qualifiers, order)
                        })
                })
        })
}

/// Whether a locked relation name comes from an interpolation, so no catalog entry can
/// exist for it and the key-order message would only mislead.
pub(super) fn locks_interpolated_relation(lock: &LockingSelectMetadata) -> bool {
    lock.tables.as_deref().is_some_and(|tables| {
        tables
            .iter()
            .any(|table| table.starts_with(INTERPOLATION_MARKER))
    })
}
