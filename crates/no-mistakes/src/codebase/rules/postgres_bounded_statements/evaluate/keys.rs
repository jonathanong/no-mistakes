//! Compile catalog and pin evidence once before bounded-state propagation.
use super::{qualified, reads_outer, Evaluation};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlPinSource};
use crate::codebase::postgres::{RelationKind, SchemaCatalog};

/// Keys, their required columns, and alternative source-item dependencies for each column.
pub(super) type Keys = Vec<Vec<Vec<Vec<usize>>>>;

pub(super) fn prepare(
    item: &SqlBoundItem,
    subqueries: &[Option<Evaluation>],
    arrays: &[bool],
    catalog: &SchemaCatalog,
) -> Keys {
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return Vec::new();
    };
    let Some(table) = catalog.relation(name) else {
        return Vec::new();
    };
    let mut keys: Vec<Vec<(String, String)>> = catalog
        .unique_keys(name)
        .into_iter()
        .filter_map(|key| {
            key.into_iter()
                .map(|physical| {
                    let visible =
                        super::super::arrays::key_visible(table, &item.column_aliases, &physical)?
                            .to_owned();
                    Some((visible, physical))
                })
                .collect()
        })
        .collect();
    if table.relation_kind != RelationKind::PartitionedTable
        && !item.column_aliases.iter().any(|alias| alias == "ctid")
    {
        keys.push(vec![("ctid".to_owned(), "ctid".to_owned())]);
    }
    // Static read/cast evidence is independent of the fixed point, including failed proofs.
    let sources: Vec<_> = item
        .pins
        .iter()
        .enumerate()
        .map(|(index, pin)| {
            if !arrays[index]
                || reads_outer(&pin.reads, catalog)
                || qualified::reads_outer(&pin.qualified_reads, catalog)
            {
                return None;
            }
            match &pin.source {
                SqlPinSource::Value => Some(Vec::new()),
                SqlPinSource::Items(items) | SqlPinSource::Array { items, .. } => {
                    Some(items.clone())
                }
                SqlPinSource::Query(_) => subqueries[index]
                    .as_ref()
                    .filter(|query| query.bounded)
                    .map(|_| Vec::new()),
                SqlPinSource::StoredArray(_) | SqlPinSource::ReadQuery(_) => None,
            }
        })
        .collect();
    keys.into_iter()
        .map(|key| {
            key.into_iter()
                .map(|(visible, physical)| {
                    let not_null = physical == "ctid"
                        || table
                            .columns
                            .iter()
                            .any(|column| column.name == physical && !column.nullable);
                    item.pins
                        .iter()
                        .zip(&sources)
                        .filter_map(|(pin, source)| {
                            (pin.column == visible && (!pin.null_safe || not_null))
                                .then(|| source.clone())
                                .flatten()
                        })
                        .collect()
                })
                .collect()
        })
        .collect()
}
