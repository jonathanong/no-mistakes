use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(super) struct CatalogLocations {
    pub(super) tables: BTreeMap<String, usize>,
    pub(super) columns: BTreeMap<(String, String), usize>,
}

pub(super) fn catalog_locations(
    value: &jsonc_parser::ast::Value<'_>,
    source: &str,
) -> CatalogLocations {
    let Some(tables) = value
        .as_object()
        .and_then(|snapshot| snapshot.get("tables"))
        .and_then(|tables| tables.value.as_object())
    else {
        return CatalogLocations::default();
    };
    let line_starts = source
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1))
        .collect::<Vec<_>>();
    let mut locations = CatalogLocations::default();
    for table in &tables.properties {
        let table_name = table.name.as_str().to_string();
        let table_line = line_starts.partition_point(|start| *start < table.range.start) + 1;
        locations.tables.insert(table_name.clone(), table_line);
        let Some(columns) = table
            .value
            .as_object()
            .and_then(|value| value.get("columns"))
            .and_then(|columns| columns.value.as_object())
        else {
            continue;
        };
        for column in &columns.properties {
            let line = line_starts.partition_point(|start| *start < column.range.start) + 1;
            locations
                .columns
                .insert((table_name.clone(), column.name.as_str().to_string()), line);
        }
    }
    locations
}
