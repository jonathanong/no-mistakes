use super::super::view_bounds::ViewBounds;
use super::{lateral, qualified, Evaluation};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use crate::codebase::postgres::SchemaCatalog;
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
        project(inner, item, catalog, views)
    } else {
        Cow::Borrowed(inner)
    };
    Some(super::evaluate(&projected, catalog, views, precise_columns))
}

fn project<'a>(
    inner: &'a SqlBoundQuery,
    parent: &SqlBoundItem,
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
        for (pin_index, pin) in item.pins.iter().enumerate() {
            if pin.expression.is_some()
                || !views.keys.columns(&table.name).any(|column| {
                    super::super::arrays::key_visible(table, &item.column_aliases, column)
                        == Some(pin.column.as_str())
                })
            {
                continue;
            }
            let allowed = |read: &crate::codebase::postgres::statements::SqlQualifiedRead| {
                parent.lateral_qualified_reads.iter().any(|outer| {
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
