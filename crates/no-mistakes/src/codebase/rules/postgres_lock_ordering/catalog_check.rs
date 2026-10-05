//! Schema-catalog proofs for one locking `SELECT`.

use crate::codebase::postgres::{LockingSelectMetadata, SchemaCatalog};
use std::collections::BTreeSet;

/// Generated interpolation marker the embedded-SQL collector substitutes for `${...}`.
const INTERPOLATION_MARKER: &str = "sql_placeholder_";

/// Whether the lock touches at most one row of every locked table.
///
/// A relation is single-row when some catalog unique key has every column pinned: by a
/// top-level `WHERE` / inner-join `ON` equality to a bound value, or to a column of a relation
/// already proven single-row (iterated to a fixpoint). An `IN` or `= ANY` elsewhere can then
/// only narrow that row, never widen it.
pub(super) fn locks_single_row(lock: &LockingSelectMetadata, catalog: &SchemaCatalog) -> bool {
    let (Some(tables), Some(pinned)) = (&lock.tables, &lock.pinned_columns) else {
        return false;
    };
    let mut proven = BTreeSet::new();
    loop {
        let before = proven.len();
        for (table, bound) in pinned {
            if !proven.contains(table) && has_pinned_key(table, bound, lock, &proven, catalog) {
                proven.insert(table.clone());
            }
        }
        if proven.len() == before {
            break;
        }
    }
    !tables.is_empty() && tables.iter().all(|table| proven.contains(table))
}

fn has_pinned_key(
    table: &str,
    bound: &[String],
    lock: &LockingSelectMetadata,
    proven: &BTreeSet<String>,
    catalog: &SchemaCatalog,
) -> bool {
    let mut pinned: BTreeSet<&str> = bound.iter().map(String::as_str).collect();
    for join in &lock.join_equalities {
        if join.left_table == table && proven.contains(&join.right_table) {
            pinned.insert(&join.left_column);
        }
        if join.right_table == table && proven.contains(&join.left_table) {
            pinned.insert(&join.right_column);
        }
    }
    catalog
        .unique_keys(table)
        .iter()
        .any(|key| !key.is_empty() && key.iter().all(|column| pinned.contains(column.as_str())))
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
