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
    for column in schema
        .iter()
        .flat_map(|file| &file.add_columns)
        .filter(|column| column.is_generated)
    {
        let name = column.unqualified_table_name.as_str();
        let order = schema
            .iter()
            .flat_map(|file| &file.tables)
            .find(|table| table.table_name.eq_ignore_ascii_case(name))
            .map(|table| {
                let mut order = table
                    .columns
                    .iter()
                    .map(|column| column.name.to_ascii_lowercase())
                    .collect::<Vec<_>>();
                for added in schema
                    .iter()
                    .flat_map(|file| &file.add_columns)
                    .filter(|added| added.unqualified_table_name.eq_ignore_ascii_case(name))
                {
                    let column = added.column_name.to_ascii_lowercase();
                    if !order.contains(&column) {
                        order.push(column);
                    }
                }
                order
            });
        catalog.insert_table(GeneratedTable {
            name: name.to_string(),
            generated: [column.column_name.to_ascii_lowercase()]
                .into_iter()
                .collect(),
            column_order: order,
        });
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

mod trigger;
pub(super) use trigger::trigger_catalog_from_facts;

pub(super) fn stale_trigger_findings(
    schema: &[SqlSchemaFileFacts],
    columns: &[String],
) -> Vec<crate::codebase::rules::RuleFinding> {
    columns
        .iter()
        .filter(|column| !schema_has_column(schema, column))
        .map(|column| crate::codebase::rules::RuleFinding {
            rule: super::RULE_ID.to_string(),
            file: ".no-mistakes.yml".to_string(),
            line: 1,
            message: format!(
                "stale triggerMaintainedColumns entry: `{column}` matches no column in schema SQL"
            ),
            import: Some(column.clone()),
            target: Some(column.clone()),
        })
        .collect()
}

fn schema_has_column(schema: &[SqlSchemaFileFacts], name: &str) -> bool {
    schema.iter().any(|file| {
        file.add_columns
            .iter()
            .any(|column| column.column_name.eq_ignore_ascii_case(name))
    }) || schema.iter().flat_map(|file| &file.tables).any(|table| {
        table
            .columns
            .iter()
            .any(|column| column.name.eq_ignore_ascii_case(name))
    })
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
    let mut generated: std::collections::BTreeSet<_> = schema
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
        .collect();
    generated.extend(
        schema
            .iter()
            .flat_map(|file| &file.add_columns)
            .filter(|column| column.is_generated)
            .map(|column| {
                (
                    column.unqualified_table_name.to_ascii_lowercase(),
                    column.column_name.to_ascii_lowercase(),
                )
            }),
    );
    generated
}

#[cfg(test)]
mod tests;
