use super::super::qualified;
use super::ViewBounds;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlQualifiedRead};
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::Expr;
use std::borrow::Cow;

pub(in super::super) fn project<'a>(
    inner: &'a SqlBoundQuery,
    outer_reads: &[SqlQualifiedRead],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Cow<'a, SqlBoundQuery> {
    let mut projected = Cow::Borrowed(inner);
    for (item_index, item) in inner.items.iter().enumerate() {
        match &item.kind {
            SqlBoundItemKind::Table(name) => {
                project_table(
                    &mut projected,
                    item_index,
                    item,
                    name,
                    outer_reads,
                    catalog,
                    views,
                );
            }
            SqlBoundItemKind::Query(nested) => {
                // Only LATERAL items carry these reads. Strip inherited drivers at
                // their own boundary, then let the child resolve its nearer drivers.
                let inherited: Vec<_> = item
                    .lateral_qualified_reads
                    .iter()
                    .filter(|read| approved(read, outer_reads, catalog))
                    .cloned()
                    .collect();
                if inherited.is_empty() {
                    continue;
                }
                if let Cow::Owned(nested) = project(nested, &inherited, catalog, views) {
                    projected.to_mut().items[item_index].kind = SqlBoundItemKind::Query(nested);
                }
                projected.to_mut().items[item_index]
                    .lateral_qualified_reads
                    .retain(|read| !approved(read, outer_reads, catalog));
            }
            _ => {}
        }
    }
    projected
}

fn project_table(
    projected: &mut Cow<'_, SqlBoundQuery>,
    item_index: usize,
    item: &crate::codebase::postgres::statements::SqlBoundItem,
    name: &str,
    outer_reads: &[SqlQualifiedRead],
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) {
    let Some(table) = catalog.bounded_relation(name) else {
        return;
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
                    super::super::super::arrays::key_visible(table, &item.column_aliases, column)
                        == Some(pin.column.as_str())
                })
        {
            continue;
        }
        if pin
            .qualified_reads
            .iter()
            .any(|read| approved(read, outer_reads, catalog))
        {
            projected.to_mut().items[item_index].pins[pin_index]
                .qualified_reads
                .retain(|read| !approved(read, outer_reads, catalog));
        }
    }
}

fn approved(
    read: &SqlQualifiedRead,
    outer_reads: &[SqlQualifiedRead],
    catalog: &SchemaCatalog,
) -> bool {
    outer_reads
        .iter()
        .any(|outer| outer.qualifier == read.qualifier && outer.scopes.starts_with(&read.scopes))
        && qualified::reads_outer(std::slice::from_ref(read), catalog)
}
