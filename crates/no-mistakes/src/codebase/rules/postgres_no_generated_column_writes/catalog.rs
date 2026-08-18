use super::ExtraGeneratedColumn;
use crate::codebase::postgres::dml::{GeneratedTable, GeneratedTableColumns};
use crate::codebase::postgres::SqlSchemaFileFacts;

pub(super) fn catalog_from_facts(
    schema: &[SqlSchemaFileFacts],
    extra: &[ExtraGeneratedColumn],
) -> GeneratedTableColumns {
    let mut catalog = GeneratedTableColumns::default();
    for file in schema {
        for table in &file.tables {
            let generated: Vec<String> = table
                .columns
                .iter()
                .filter(|column| column.is_generated)
                .map(|column| column.name.to_ascii_lowercase())
                .collect();
            if generated.is_empty() {
                continue;
            }
            catalog.insert_table(GeneratedTable {
                name: table.table_name.clone(),
                generated: generated.into_iter().collect(),
                column_order: Some(
                    table
                        .columns
                        .iter()
                        .map(|column| column.name.to_ascii_lowercase())
                        .collect(),
                ),
            });
        }
    }
    for extra in extra {
        if extra.table.is_empty() || extra.column.is_empty() {
            continue;
        }
        catalog.insert_table(GeneratedTable {
            name: extra.table.clone(),
            generated: [extra.column.to_ascii_lowercase()].into_iter().collect(),
            column_order: None,
        });
    }
    catalog
}

#[cfg(test)]
mod tests;
