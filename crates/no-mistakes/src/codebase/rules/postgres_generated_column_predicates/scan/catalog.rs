use super::super::{CompiledOptions, ExtraGeneratedColumn, RuleFinding, RULE_ID};
use crate::codebase::postgres::SqlSchemaFileFacts;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Tracked {
    pub(super) table: String,
    pub(super) column: String,
    pub(super) source: String,
    pub(super) function: String,
}

pub(super) fn tracked_columns(
    schema: &[SqlSchemaFileFacts],
    opts: &CompiledOptions,
) -> Vec<Tracked> {
    let mut tracked = Vec::new();
    for file in schema {
        for table in &file.tables {
            let name = table.table_name.to_ascii_lowercase();
            let keys: Vec<String> = table
                .columns
                .iter()
                .filter(|column| column.is_primary_key)
                .map(|column| column.name.to_ascii_lowercase())
                .collect();
            for column in &table.columns {
                let Some(function) = column.generated_function.as_deref() else {
                    continue;
                };
                if !opts
                    .functions
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(function))
                {
                    continue;
                }
                let args = &column.generated_function_arg_columns;
                if args.len() != 1 {
                    continue;
                }
                let source = args[0].to_ascii_lowercase();
                if opts.require_primary_key && !(keys.len() == 1 && keys[0] == source) {
                    continue;
                }
                tracked.push(Tracked {
                    table: name.clone(),
                    column: column.name.to_ascii_lowercase(),
                    source,
                    function: function.to_ascii_lowercase(),
                });
            }
        }
    }
    let function = opts.functions.first().cloned().unwrap_or_default();
    for extra in &opts.extras {
        let table = extra.table.trim().to_ascii_lowercase();
        let column = extra.column.trim().to_ascii_lowercase();
        if tracked
            .iter()
            .any(|item| item.table == table && item.column == column)
        {
            continue;
        }
        tracked.push(Tracked {
            table,
            column,
            source: extra.source_column.trim().to_ascii_lowercase(),
            function: function.clone(),
        });
    }
    tracked
}

pub(super) fn column_index(schema: &[SqlSchemaFileFacts]) -> BTreeMap<String, BTreeSet<String>> {
    let mut columns = BTreeMap::new();
    for file in schema {
        for table in &file.tables {
            columns.insert(
                table.table_name.to_ascii_lowercase(),
                table
                    .columns
                    .iter()
                    .map(|column| column.name.to_ascii_lowercase())
                    .collect(),
            );
        }
    }
    columns
}

pub(super) fn stale_extras(
    schema: &[SqlSchemaFileFacts],
    extras: &[ExtraGeneratedColumn],
) -> Vec<RuleFinding> {
    extras
        .iter()
        .filter(|extra| generated_in_schema(schema, extra))
        .map(|extra| RuleFinding {
            rule: RULE_ID.to_string(),
            file: ".no-mistakes.yml".to_string(),
            line: 1,
            message: format!(
                "stale postgres-generated-column-predicates extraGeneratedColumns entry: {}.{}",
                extra.table.trim(),
                extra.column.trim()
            ),
            import: Some(format!("{}.{}", extra.table.trim(), extra.column.trim())),
            target: Some(extra.column.trim().to_string()),
        })
        .collect()
}

fn generated_in_schema(schema: &[SqlSchemaFileFacts], extra: &ExtraGeneratedColumn) -> bool {
    schema.iter().flat_map(|file| &file.tables).any(|table| {
        table.table_name.eq_ignore_ascii_case(extra.table.trim())
            && table.columns.iter().any(|column| {
                column.is_generated && column.name.eq_ignore_ascii_case(extra.column.trim())
            })
    })
}
