use super::{correlated, ViewBounds};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundQuery, SqlQualifiedRead};
use std::borrow::Cow;

pub(super) fn project<'a>(
    query: &'a SqlBoundQuery,
    reads: &[SqlQualifiedRead],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Cow<'a, SqlBoundQuery> {
    let approved: Vec<_> = reads
        .iter()
        .filter(|read| {
            super::qualified::reads_outer(std::slice::from_ref(read), catalog)
                && read.scopes.iter().all(|scope| {
                    !scope.unknown
                        && scope
                            .tables
                            .iter()
                            .all(|table| catalog.relation(table).is_some())
                })
        })
        .cloned()
        .collect();
    if approved.is_empty() {
        Cow::Borrowed(query)
    } else {
        correlated::project(query, &approved, catalog, views)
    }
}
