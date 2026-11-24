use super::super::super::statements::CatalogInvalidations;
use super::super::unique::CatalogContext;
use crate::codebase::postgres::statements::lock_writes::{LockWriteFacts, ShareStatement};
use crate::codebase::postgres::EmbeddedSqlCall;
use std::collections::BTreeSet;

pub(super) struct KeyWrites<'a, 'facts> {
    pub(super) catalog: CatalogContext<'a, 'facts>,
    pub(super) unresolved: &'a BTreeSet<u32>,
}

impl KeyWrites<'_, '_> {
    pub(super) fn matches(
        &self,
        call: &EmbeddedSqlCall,
        access: &LockWriteFacts,
        statement: &ShareStatement,
        index: usize,
    ) -> bool {
        if statement.deletes.binary_search(&index).is_ok() {
            return true;
        }
        let (Some(relation), Some(columns)) = (
            access.writes[index].relation.as_deref(),
            statement.update_columns.get(&index),
        ) else {
            return false;
        };
        CatalogInvalidations::for_lock(self.catalog, call, Some(relation), self.unresolved)
            .and_then(|catalog| catalog.relation(relation))
            .is_some_and(|table| {
                table.indexes.iter().any(|index| {
                    index.unique
                        && index.valid
                        && index.ready
                        && index.live
                        && index.immediate
                        && index.predicate.is_none()
                        && index.keys.iter().all(|key| key.column.is_some())
                        && index.keys.iter().any(|key| {
                            key.column
                                .as_ref()
                                .is_some_and(|column| columns.contains(column))
                        })
                })
            })
    }
}
