use super::super::rows;
use super::super::statements::CatalogInvalidations;
use crate::codebase::postgres::statements::lock_writes::RowAccess;
use crate::codebase::postgres::{CatalogCoverage, EmbeddedSqlCall, SchemaCatalog};
use std::collections::BTreeSet;

pub(in super::super) type CatalogContext<'a, 'facts> =
    Option<(&'a SchemaCatalog, &'a CatalogInvalidations<'facts>)>;

pub(super) fn key(
    context: CatalogContext<'_, '_>,
    call: &EmbeddedSqlCall,
    lock: &RowAccess,
    unresolved: &BTreeSet<u32>,
) -> bool {
    let (Some(relation), Some((column, _))) = (lock.relation.as_deref(), &lock.key) else {
        return false;
    };
    CatalogInvalidations::for_lock(context, call, Some(relation), unresolved).is_some_and(
        |catalog| {
            catalog.coverage() == CatalogCoverage::Complete
                && catalog.columns_pin_one_row(relation, std::slice::from_ref(column))
        },
    )
}

pub(super) fn lock_scope(
    context: CatalogContext<'_, '_>,
    call: &EmbeddedSqlCall,
    lock: &RowAccess,
    unresolved: &BTreeSet<u32>,
    only: bool,
    upsert: bool,
) -> Result<(), &'static str> {
    if only {
        return Ok(());
    }
    if !upsert {
        return Err("cannot prove the pre-lock excludes inherited rows; use FROM ONLY when the lock should target just this table");
    }
    let proven = lock.relation.as_deref().is_some_and(|relation| {
        let catalog = CatalogInvalidations::for_lock(context, call, Some(relation), unresolved);
        rows::upsert_lock_scope(false, catalog, relation)
    });
    proven.then_some(()).ok_or(rows::UNCERTAIN_UPSERT_SCOPE)
}
