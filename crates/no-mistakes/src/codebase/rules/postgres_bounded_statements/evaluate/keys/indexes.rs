use super::Keys;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::SqlBoundItem;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use sqlparser::ast::Expr;
use std::collections::BTreeMap;

pub(super) fn append(
    prepared: &mut Keys,
    item: &SqlBoundItem,
    item_index: usize,
    owners: &BTreeMap<String, usize>,
    (name, table): (&str, &CatalogTable),
    sources: &[Option<Vec<usize>>],
    catalog: &SchemaCatalog,
) {
    for index in catalog.bounded_unique_indexes(name) {
        // Positional aliases can exchange physical predicate columns while retaining the same
        // visible spelling. Without an ordinal projection, no partial predicate is proved.
        if !index.predicate.is_empty() && !item.column_aliases.is_empty() {
            continue;
        }
        if !index
            .predicate
            .iter()
            .all(|predicate| implied(predicate, item))
        {
            continue;
        }
        let key: Option<Vec<_>> = index
            .keys
            .iter()
            .map(|expression| {
                let expression = unwrap_expr(expression);
                let physical = match expression {
                    Expr::Identifier(ident) => Some(ident_key(ident)),
                    _ if item.column_aliases.is_empty() => None,
                    _ => return None,
                };
                let visible = physical.as_deref().map(|physical| {
                    super::super::super::arrays::key_visible(table, &item.column_aliases, physical)
                });
                let visible = match visible {
                    Some(value) => Some(value?),
                    None => None,
                };
                let not_null = physical.as_deref().is_some_and(|physical| {
                    table
                        .columns
                        .iter()
                        .any(|column| column.name == physical && !column.nullable)
                });
                let matching = item
                    .pins
                    .iter()
                    .zip(sources)
                    .filter_map(|(pin, source)| {
                        let matches = match &visible {
                            Some(visible) => pin.expression.is_none() && pin.column == *visible,
                            None => pin.expression.as_ref() == Some(expression),
                        };
                        (matches
                            && (!pin.unqualified_candidate
                                || owners.get(&pin.column) == Some(&item_index))
                            && (!pin.null_safe || not_null || index.nulls_not_distinct))
                            .then(|| source.clone())
                            .flatten()
                    })
                    .collect();
                Some(matching)
            })
            .collect();
        if let Some(key) = key {
            prepared.push(key);
        }
    }
}

fn implied(predicate: &Expr, item: &SqlBoundItem) -> bool {
    if item.predicate_conjuncts.contains(predicate) {
        return true;
    }
    match unwrap_expr(predicate) {
        Expr::IsNotNull(column) => match unwrap_expr(column) {
            Expr::Identifier(ident) => item.nonnull_columns.contains(&ident_key(ident)),
            _ => false,
        },
        _ => false,
    }
}
