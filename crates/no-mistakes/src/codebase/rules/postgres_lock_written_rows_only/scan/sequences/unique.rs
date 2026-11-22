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
