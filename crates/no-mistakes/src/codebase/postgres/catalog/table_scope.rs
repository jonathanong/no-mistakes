//! Preserve the rolled-up relation universe for rules unrelated to leaf-local keys.
use super::{CatalogTable, RelationKind, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn logical_tables(&self) -> impl Iterator<Item = &CatalogTable> {
        self.tables().filter(|table| {
            table.partition_of.is_none() || table.relation_kind == RelationKind::PartitionedTable
        })
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
