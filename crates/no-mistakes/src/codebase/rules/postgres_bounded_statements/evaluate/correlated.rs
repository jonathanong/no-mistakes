use super::super::view_bounds::ViewBounds;
use super::{lateral, qualified, Evaluation};
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource, SqlQualifiedRead,
};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::Expr;
use std::borrow::Cow;

pub(super) fn nested(
    item: &SqlBoundItem,
    preceding: &[SqlBoundItem],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Option<Evaluation> {
    let SqlBoundItemKind::Query(inner) = &item.kind else {
        return None;
    };
    let projected = if lateral::dependencies(item, preceding, catalog).is_some() {
        project(inner, &item.lateral_qualified_reads, catalog, views)
    } else {
        Cow::Borrowed(inner)
    };
    Some(super::evaluate(
        &projected,
        catalog,
        views,
        precise_columns,
        None,
    ))
}

pub(super) fn pin_query(
    inner: &SqlBoundQuery,
    pin: &SqlBoundPin,
    outer: &[SqlBoundItem],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
) -> Evaluation {
    let projected = if !pin.qualified_reads.is_empty()
        && !matches!(pin.source, SqlPinSource::ReadQuery(_))
        && lateral::read_dependencies(&pin.reads, &pin.qualified_reads, outer, catalog).is_some()
    {
        project(inner, &pin.qualified_reads, catalog, views)
    } else {
        Cow::Borrowed(inner)
    };
    super::evaluate(&projected, catalog, views, precise_columns, None)
}

fn project<'a>(
    inner: &'a SqlBoundQuery,
    outer_reads: &[SqlQualifiedRead],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Cow<'a, SqlBoundQuery> {
    let mut projected = Cow::Borrowed(inner);
    for (item_index, item) in inner.items.iter().enumerate() {
        let SqlBoundItemKind::Table(name) = &item.kind else {
            continue;
        };
        let Some(table) = catalog.bounded_relation(name) else {
            continue;
        };
        let partial = catalog
            .bounded_unique_indexes(name)
            .iter()
            .flat_map(|index| index.keys.iter())
            .filter_map(|key| match unwrap_expr(key) {
                Expr::Identifier(ident) => Some(ident_key(ident)),
                _ => None,
            });
        let unique_columns: Vec<String> = catalog
            .unique_keys(name)
            .into_iter()
            .flatten()
            .chain(partial)
            .collect();
        for (pin_index, pin) in item.pins.iter().enumerate() {
            if pin.expression.is_some()
                || !views
                    .keys
                    .columns(&table.name)
                    .chain(unique_columns.iter().map(String::as_str))
                    .any(|column| {
                        super::super::arrays::key_visible(table, &item.column_aliases, column)
                            == Some(pin.column.as_str())
                    })
            {
                continue;
            }
            let allowed = |read: &crate::codebase::postgres::statements::SqlQualifiedRead| {
                outer_reads.iter().any(|outer| {
                    outer == read || (read.scopes.is_empty() && outer.qualifier == read.qualifier)
                }) && qualified::reads_outer(std::slice::from_ref(read), catalog)
            };
            if pin.qualified_reads.iter().any(allowed) {
                projected.to_mut().items[item_index].pins[pin_index]
                    .qualified_reads
                    .retain(|read| !allowed(read));
            }
        }
    }
    projected
}
