//! Close item bounds, then decide whether grouping makes the result bounded.
use super::super::view_bounds::ViewBounds;
use super::keys::Keys;
use super::{propagation, reads, singleton, Evaluation, Offender};
use crate::codebase::postgres::statements::SqlBoundQuery;
use crate::codebase::postgres::SchemaCatalog;

pub(super) fn finish(
    query: &SqlBoundQuery,
    keys: &[Keys],
    nested: &[Option<Evaluation>],
    pin_subqueries: &[Vec<Option<Evaluation>>],
    bounded: &mut [bool],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> (bool, Vec<Offender>) {
    propagation::bound(keys, bounded);
    singleton::finish(query, catalog, keys, bounded);
    let capped = query.capped || query.group_constant || grouped(query, bounded);
    let offenders = reads::collect(
        query,
        nested,
        pin_subqueries,
        bounded,
        catalog,
        views,
        precise_columns,
        capped,
    );
    (capped || bounded.iter().all(|state| *state), offenders)
}

/// Every plain GROUP BY key is one value or comes from an item already bounded.
fn grouped(query: &SqlBoundQuery, bounded: &[bool]) -> bool {
    !query.group_keys.is_empty()
        && query.group_keys.iter().all(|key| {
            key.constant
                || key
                    .item
                    .is_some_and(|item| bounded.get(item).copied().unwrap_or(false))
        })
}
