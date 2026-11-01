//! Pin evidence for a view whose definition projects a base primary key.
use super::Keys;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlPinSource};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeMap;

pub(super) fn keys_for(
    item: &SqlBoundItem,
    item_index: usize,
    owners: &BTreeMap<String, usize>,
    subqueries: &[Option<super::super::Evaluation>],
    arrays: &[bool],
    catalog: &SchemaCatalog,
    name: &str,
) -> Keys {
    let Some(key) = super::super::view_key::visible_key(catalog, name) else {
        return Vec::new();
    };
    if !item.column_aliases.is_empty()
        && !key.iter().all(|column| {
            item.column_aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(column))
        })
    {
        return Vec::new();
    }
    let sources: Vec<_> = item
        .pins
        .iter()
        .enumerate()
        .map(|(index, pin)| {
            if !arrays[index]
                || super::super::reads_outer(&pin.reads, catalog)
                || super::super::qualified::reads_outer(&pin.qualified_reads, catalog)
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
                SqlPinSource::ArrayQuery(query) => subqueries[index]
                    .as_ref()
                    .filter(|evaluation| {
                        evaluation.bounded && super::super::scalar_output::single(query, catalog)
                    })
                    .map(|_| Vec::new()),
                SqlPinSource::StoredArray(_) | SqlPinSource::ReadQuery(_) => None,
            }
        })
        .collect();
    (0..item.pin_alternatives.max(1))
        .map(|alternative| {
            vec![key
                .iter()
                .map(|column| {
                    item.pins
                        .iter()
                        .zip(&sources)
                        .filter_map(|(pin, source)| {
                            (pin.column.eq_ignore_ascii_case(column)
                                && pin.alternative.is_none_or(|branch| branch == alternative)
                                && pin.expression.is_none()
                                && (!pin.unqualified_candidate
                                    || owners.get(&pin.column) == Some(&item_index)))
                            .then(|| source.clone())
                            .flatten()
                        })
                        .collect()
                })
                .collect()]
        })
        .collect()
}
