//! A PK-keyed target of an opaque recordset is not itself a root.
use super::Offender;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SchemaCatalog;

pub(super) fn roots(
    query: &SqlBoundQuery,
    target: usize,
    offenders: &[Offender],
    catalog: &SchemaCatalog,
) -> Option<Vec<Offender>> {
    let item = query.items.get(target)?;
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return None;
    };
    if !explained(item, query, catalog) {
        return None;
    }
    let roots = offenders
        .iter()
        .filter(|offender| offender.table != *name)
        .cloned()
        .collect::<Vec<_>>();
    (!roots.is_empty()).then_some(roots)
}

pub(super) fn suppress(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    offenders: &mut Vec<Offender>,
) {
    let names = query
        .items
        .iter()
        .filter(|item| explained(item, query, catalog))
        .filter_map(|item| match &item.kind {
            SqlBoundItemKind::Table(name) => Some(name.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        return;
    }
    let kept = offenders
        .iter()
        .filter(|offender| !names.iter().any(|name| name == &offender.table))
        .cloned()
        .collect::<Vec<_>>();
    if !kept.is_empty() {
        *offenders = kept;
    }
}

fn explained(item: &SqlBoundItem, query: &SqlBoundQuery, catalog: &SchemaCatalog) -> bool {
    if item.pin_alternatives > 1 {
        return false;
    }
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return false;
    };
    catalog.unique_keys(name).into_iter().any(|key| {
        !key.is_empty()
            && key.iter().all(|column| {
                item.pins.iter().any(|pin| {
                    pin.column == *column
                        && pin.alternative.is_none()
                        && pin.expression.is_none()
                        && !pin.unqualified_candidate
                        && opaque_queries(&pin.source, query)
                })
            })
    })
}

fn opaque_queries(source: &SqlPinSource, query: &SqlBoundQuery) -> bool {
    let SqlPinSource::Items(indexes) = source else {
        return false;
    };
    !indexes.is_empty()
        && indexes.iter().all(|index| {
            matches!(
                query.items.get(*index).map(|item| &item.kind),
                Some(SqlBoundItemKind::Query(inner)) if recordset_carrier(inner)
            )
        })
}

/// A select-list SRF sits beside the scanned tables and does not size them.
/// `alias_explicit` with no alias is that marker. A lone opaque item is a
/// recursive placeholder, not a recordset fed by a relation.
fn recordset_carrier(query: &SqlBoundQuery) -> bool {
    if query.items.iter().any(projection_expander) {
        return false;
    }
    let carries = |item: &SqlBoundItem| match &item.kind {
        SqlBoundItemKind::Opaque => true,
        SqlBoundItemKind::Query(inner) => recordset_carrier(inner),
        _ => false,
    };
    query.items.iter().any(carries)
        && query.items.iter().any(|item| {
            matches!(item.kind, SqlBoundItemKind::Table(_))
                || matches!(&item.kind, SqlBoundItemKind::Query(inner) if recordset_carrier(inner))
        })
}

fn projection_expander(item: &SqlBoundItem) -> bool {
    matches!(item.kind, SqlBoundItemKind::Opaque) && item.alias.is_none() && item.alias_explicit
}
