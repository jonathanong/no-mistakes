use super::{schema_generated, GeneratedTable, GeneratedTableColumns, SqlSchemaFileFacts};
use std::collections::BTreeMap;

pub(crate) fn trigger_catalog_from_facts(
    schema: &[SqlSchemaFileFacts],
    columns: &[String],
) -> GeneratedTableColumns {
    let generated = schema_generated(schema);
    let known: std::collections::BTreeSet<_> = schema
        .iter()
        .flat_map(|file| &file.tables)
        .map(|table| table.table_name.to_ascii_lowercase())
        .collect();
    let mut tables: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for file in schema {
        for table in &file.tables {
            let order = tables
                .entry(table.table_name.to_ascii_lowercase())
                .or_insert_with(|| (table.table_name.clone(), Vec::new()));
            let order = &mut order.1;
            for column in &table.columns {
                let name = column.name.to_ascii_lowercase();
                if !order.contains(&name) {
                    order.push(name);
                }
            }
        }
        for column in &file.add_columns {
            let order = tables
                .entry(column.table_name.to_ascii_lowercase())
                .or_insert_with(|| (column.table_name.clone(), Vec::new()));
            let order = &mut order.1;
            let name = column.column_name.to_ascii_lowercase();
            if !order.contains(&name) {
                order.push(name);
            }
        }
    }
    let mut catalog = GeneratedTableColumns::default();
    for (key, (table, order)) in tables {
        let maintained = order
            .iter()
            .filter(|name| {
                columns
                    .iter()
                    .any(|column| column.eq_ignore_ascii_case(name))
                    && !generated.contains(&(key.clone(), (*name).clone()))
            })
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if !maintained.is_empty() {
            catalog.insert_table(GeneratedTable {
                name: table,
                generated: maintained,
                column_order: known.contains(&key).then_some(order),
            });
        }
    }
    catalog
}
