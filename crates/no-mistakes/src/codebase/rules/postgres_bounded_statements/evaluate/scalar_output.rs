//! Catalog-proven scalar outputs prevent an array-valued row from hiding arbitrary elements.
use crate::codebase::postgres::{SchemaCatalog, SqlBoundItemKind, SqlBoundQuery};

pub(super) fn single(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> bool {
    query.outputs.len() == 1 && scalar(query, 0, catalog)
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
                .filter(|(index, output)| {
                    item.column_aliases.get(*index).or(output.name.as_ref()) == Some(name)
                })
                .map(|(index, _)| index)
                .collect();
            matches.len() == 1 && scalar(inner, matches[0], catalog)
        }
        _ => false,
    }
}
