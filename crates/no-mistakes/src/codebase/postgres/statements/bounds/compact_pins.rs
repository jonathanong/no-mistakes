//! Pins retained when compaction discards the original FROM scope.
use crate::codebase::postgres::SqlPinSource;

pub(super) fn independent(source: &SqlPinSource) -> bool {
    match source {
        SqlPinSource::Value => true,
        SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            ..
        } => {
            // Cast evidence stays on the pin for catalog validation; no row evidence may remain.
            [items.len(), scalar_columns.len(), indexed_columns.len()] == [0; 3]
        }
        _ => false,
    }
}
