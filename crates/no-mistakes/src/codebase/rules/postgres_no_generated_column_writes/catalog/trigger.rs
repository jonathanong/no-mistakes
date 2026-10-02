use super::{GeneratedTable, GeneratedTableColumns, LiveTables};

pub(crate) fn trigger_catalog_from_tables(
    tables: &LiveTables<'_>,
    columns: &[String],
) -> GeneratedTableColumns {
    let mut catalog = super::empty_catalog_for_tables(tables);
    for (key, table) in tables {
        let maintained = table
            .columns
            .iter()
            .filter(|column| {
                !column.generated
                    && columns
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(column.name))
            })
            .map(|column| column.name.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if !maintained.is_empty() {
            catalog.insert_table_with_key(
                key,
                GeneratedTable {
                    name: catalog.display_name(key, table.name),
                    generated: maintained,
                    column_order: table.complete.then(|| {
                        table
                            .columns
                            .iter()
                            .map(|column| column.name.to_ascii_lowercase())
                            .collect()
                    }),
                },
            );
        }
    }
    catalog
}
