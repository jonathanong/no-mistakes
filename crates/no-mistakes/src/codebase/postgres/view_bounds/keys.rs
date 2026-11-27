use crate::codebase::postgres::{CatalogIndexInfo, CatalogTable};

pub(super) fn indexed_column(table: &CatalogTable, column: &str) -> bool {
    constraint_column(table, column)
        || table
            .indexes
            .iter()
            .any(|index| index_column(index, column))
}

pub(super) fn key_column(table: &CatalogTable, column: &str) -> bool {
    constraint_column(table, column)
        || table
            .indexes
            .iter()
            .any(|index| index.unique && index_column(index, column))
}

fn constraint_column(table: &CatalogTable, column: &str) -> bool {
    table
        .primary_key
        .as_ref()
        .is_some_and(|key| first(key, column))
        || table
            .unique_constraints
            .iter()
            .any(|key| first(&key.columns, column))
}

fn first(columns: &[String], column: &str) -> bool {
    columns.first().is_some_and(|key| {
        key == column || *key == crate::codebase::postgres::idents::relation_part_key(column)
    })
}

fn index_column(index: &CatalogIndexInfo, column: &str) -> bool {
    index.valid
        && index.ready
        && index.live
        && index.predicate.is_none()
        && index.access_method.eq_ignore_ascii_case("btree")
        && index.keys.first().and_then(|key| key.column.as_deref()) == Some(column)
}
