//! Preserve the rolled-up relation universe for rules unrelated to leaf-local keys.
use super::{CatalogTable, RelationKind, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn logical_tables(&self) -> impl Iterator<Item = &CatalogTable> {
        self.tables().filter(|table| {
            table.partition_of.is_none() || table.relation_kind == RelationKind::PartitionedTable
        })
    }
}

#[cfg(test)]
mod tests;
