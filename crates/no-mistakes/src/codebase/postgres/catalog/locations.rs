use std::collections::BTreeMap;

pub(super) fn column_lines(
    value: &jsonc_parser::ast::Value<'_>,
    source: &str,
) -> BTreeMap<(String, String), usize> {
    let Some(tables) = value
        .as_object()
        .and_then(|snapshot| snapshot.get("tables"))
        .and_then(|tables| tables.value.as_object())
    else {
        return BTreeMap::new();
    };
    let line_starts = source
        .bytes()
        .enumerate()
        .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1))
        .collect::<Vec<_>>();
    let mut lines = BTreeMap::new();
    for table in &tables.properties {
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
            lines.insert(
                (
                    table.name.as_str().to_string(),
                    column.name.as_str().to_string(),
                ),
                line,
            );
        }
    }
    lines
}
