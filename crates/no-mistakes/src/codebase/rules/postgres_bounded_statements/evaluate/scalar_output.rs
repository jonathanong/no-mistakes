//! Catalog-proven scalar outputs prevent an array-valued row from hiding arbitrary elements.
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundOutput};
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind, SqlBoundQuery};

pub(super) fn single(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> bool {
    !query.has_opaque_row_inputs() && query.outputs.len() == 1 && scalar(query, 0, catalog)
}

fn scalar(query: &SqlBoundQuery, ordinal: usize, catalog: &SchemaCatalog) -> bool {
    let Some(output) = query.outputs.get(ordinal) else {
        return false;
    };
    if output.scalar {
        return true;
    }
    let Some((index, name)) = &output.source_column else {
        return false;
    };
    let Some(item) = query.items.get(*index) else {
        return false;
    };
    match &item.kind {
        SqlBoundItemKind::Table(table) => catalog.relation(table).is_some_and(|table| {
            super::super::arrays::column_type(table, &item.column_aliases, name)
                .is_some_and(|column| super::super::arrays::scalar_type(&column.data_type, catalog))
        }),
        SqlBoundItemKind::Query(inner) => {
            let matches: Vec<_> = inner
                .outputs
                .iter()
                .enumerate()
                .filter(|(index, output)| exposed_name(item, *index, output) == Some(name.as_str()))
                .map(|(index, _)| index)
                .collect();
            matches.len() == 1 && scalar(inner, matches[0], catalog)
        }
        _ => false,
    }
}

/// A column alias or a stored name wins. `a.id` keeps no name, so the source column is
/// what a later `SELECT id` reads, without making the projection a complete column list.
fn exposed_name<'a>(
    item: &'a SqlBoundItem,
    index: usize,
    output: &'a SqlBoundOutput,
) -> Option<&'a str> {
    if let Some(alias) = item.column_aliases.get(index) {
        return Some(alias.as_str());
    }
    output.name.as_deref().or_else(|| {
        output
            .source_column
            .as_ref()
            .map(|(_, column)| column.as_str())
    })
}
