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

pub(super) fn stale_extra_findings(
    schema: &[SqlSchemaFileFacts],
    extras: &[ExtraGeneratedColumn],
) -> Vec<crate::codebase::rules::RuleFinding> {
    let in_schema = schema_generated(schema);
    extras
        .iter()
        .filter(|extra| !extra.table.is_empty() && !extra.column.is_empty())
        .filter(|extra| {
            in_schema.contains(&(
                extra.table.to_ascii_lowercase(),
                extra.column.to_ascii_lowercase(),
            ))
        })
        .map(|extra| crate::codebase::rules::RuleFinding {
            rule: super::RULE_ID.to_string(),
            file: ".no-mistakes.yml".to_string(),
            line: 1,
            message: format!(
                "stale extraGeneratedColumns entry: `{}.{}` is already a generated column in schema SQL",
                extra.table, extra.column
            ),
            import: Some(format!("{}.{}", extra.table, extra.column)),
            target: Some(extra.column.clone()),
        })
        .collect()
}

fn schema_generated(schema: &[SqlSchemaFileFacts]) -> std::collections::BTreeSet<(String, String)> {
    schema
        .iter()
        .flat_map(|file| &file.tables)
        .flat_map(|table| {
            table
                .columns
                .iter()
                .filter(|column| column.is_generated)
                .map(|column| {
                    (
                        table.table_name.to_ascii_lowercase(),
                        column.name.to_ascii_lowercase(),
                    )
                })
        })
        .collect()
}

#[cfg(test)]
mod tests;
