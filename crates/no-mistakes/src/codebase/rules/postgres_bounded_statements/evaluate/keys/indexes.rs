use super::predicate;
use super::Keys;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::BoundedUniqueIndex;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;
use std::collections::BTreeMap;

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
            if !predicate::holds(&index.predicate, item, alternative, &[]) {
                continue;
            }
            if let Some(key) = pinned_key(index, &pins, alternative) {
                branch.push(key);
            }
        }
    }
    // See docs/rules/postgres-bounded-statements-proof.md for the enum label cap.
    for (alternative, branch) in prepared.iter_mut().enumerate() {
        if enum_covers(name, &pins, alternative, catalog) {
            branch.push(vec![vec![vec![]]]);
        }
    }
}

fn enum_covers(name: &str, pins: &Pins<'_>, alternative: usize, catalog: &SchemaCatalog) -> bool {
    let Some(assignments) = predicate::assignments(pins.item, alternative, pins.table, catalog)
    else {
        return false;
    };
    let indexes: Vec<_> = catalog
        .bounded_unique_indexes(name)
        .iter()
        .filter(|index| index.predicate.is_empty() || pins.item.column_aliases.is_empty())
        .collect();
    assignments.iter().all(|assignment| {
        assignment.proven_empty
            || indexes.iter().any(|index| {
                predicate::holds(&index.predicate, pins.item, alternative, &assignment.extra)
                    && fully_pinned(index, pins, alternative)
            })
    })
}

fn fully_pinned(index: &BoundedUniqueIndex, pins: &Pins<'_>, alternative: usize) -> bool {
    pinned_key(index, pins, alternative)
        .is_some_and(|key| key.iter().all(|column| !column.is_empty()))
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
