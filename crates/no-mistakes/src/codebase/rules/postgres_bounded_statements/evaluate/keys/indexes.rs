use super::predicate;
use super::Keys;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::BoundedUniqueIndex;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;
use std::collections::BTreeMap;

#[path = "enum_cover.rs"]
mod enum_cover;

struct Pins<'a> {
    item: &'a SqlBoundItem,
    item_index: usize,
    owners: &'a BTreeMap<String, usize>,
    table: &'a CatalogTable,
    sources: &'a [Option<Vec<usize>>],
}

pub(super) fn append(
    prepared: &mut Keys,
    item: &SqlBoundItem,
    item_index: usize,
    owners: &BTreeMap<String, usize>,
    (name, table): (&str, &CatalogTable),
    sources: &[Option<Vec<usize>>],
    catalog: &SchemaCatalog,
) {
    let pins = Pins {
        item,
        item_index,
        owners,
        table,
        sources,
    };
    for index in catalog.bounded_unique_indexes(name) {
        // Positional aliases can exchange physical predicate columns while retaining the same
        // visible spelling. Without an ordinal projection, no partial predicate is proved.
        if !index.predicate.is_empty() && !item.column_aliases.is_empty() {
            continue;
        }
        for (alternative, branch) in prepared.iter_mut().enumerate() {
            if !predicate::holds(&index.predicate, item, alternative, &[], table, catalog) {
                continue;
            }
            if let Some(key) = pinned_key(index, &pins, alternative) {
                branch.push(key);
            }
        }
    }
    for (alternative, branch) in prepared.iter_mut().enumerate() {
        if let Some(keys) = enum_cover::keys(name, &pins, alternative, catalog) {
            branch.extend(keys);
        }
    }
}

fn pinned_key(
    index: &BoundedUniqueIndex,
    pins: &Pins<'_>,
    alternative: usize,
) -> Option<Vec<Vec<Vec<usize>>>> {
    index
        .keys
        .iter()
        .map(|expression| {
            let expression = unwrap_expr(expression);
            let physical = match expression {
                Expr::Identifier(ident) => Some(ident_key(ident)),
                _ if pins.item.column_aliases.is_empty() => None,
                _ => return None,
            };
            let visible = physical.as_deref().map(|physical| {
                super::super::super::arrays::key_visible(
                    pins.table,
                    &pins.item.column_aliases,
                    physical,
                )
            });
            let visible = match visible {
                Some(value) => Some(value?),
                None => None,
            };
            let implied = physical
                .is_none()
                .then(|| super::implied::source_column(expression, pins.table))
                .flatten();
            let not_null = physical
                .as_deref()
                .or(implied.as_deref())
                .is_some_and(|name| {
                    pins.table
                        .columns
                        .iter()
                        .any(|column| column.name == name && !column.nullable)
                });
            let matching = pins
                .item
                .pins
                .iter()
                .zip(pins.sources)
                .filter_map(|(pin, source)| {
                    let matches = match &visible {
                        Some(visible) => pin.expression.is_none() && pin.column == *visible,
                        None => {
                            pin.expression.as_ref() == Some(expression)
                                || implied.as_ref().is_some_and(|column| {
                                    pin.expression.is_none() && &pin.column == column
                                })
                        }
                    };
                    (matches
                        && (!pin.unqualified_candidate
                            || pins.owners.get(&pin.column) == Some(&pins.item_index))
                        && pin.alternative.is_none_or(|owner| owner == alternative)
                        && (!pin.null_safe || not_null || index.nulls_not_distinct))
                        .then(|| source.clone())
                        .flatten()
                })
                .collect();
            Some(matching)
        })
        .collect()
}
