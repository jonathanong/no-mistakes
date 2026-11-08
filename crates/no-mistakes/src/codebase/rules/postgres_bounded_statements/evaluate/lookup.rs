use super::{keys, ownership, propagation};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlPinSource};
use crate::codebase::postgres::SchemaCatalog;

pub(crate) fn bound(items: &mut [SqlBoundItem], bounded: &mut [bool], catalog: &SchemaCatalog) {
    for item in items.iter_mut() {
        item.pins.retain(|pin| {
            matches!(&pin.source, SqlPinSource::Items(sources) if !sources.is_empty())
                && pin.column != "ctid"
                && pin.expression.is_none()
                && pin.reads.is_empty()
                && pin.qualified_reads.is_empty()
        });
        item.predicate_conjuncts.clear();
        item.nonnull_columns.clear();
        item.rejected_alternatives.clear();
        if !matches!(item.kind, SqlBoundItemKind::Table(_)) {
            item.pins.clear();
        }
    }
    let owners = ownership::owners(items, catalog);
    let views = super::super::view_bounds::ViewBounds::default();
    let keys: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            keys::prepare(
                item,
                index,
                (&owners, None),
                &item.pins.iter().map(|_| None).collect::<Vec<_>>(),
                &vec![true; item.pins.len()],
                catalog,
                &views,
            )
        })
        .collect();
    propagation::bound(&keys, bounded);
}
