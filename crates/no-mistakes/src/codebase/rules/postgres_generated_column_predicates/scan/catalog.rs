use super::super::{CompiledOptions, ExtraGeneratedColumn, RuleFinding, RULE_ID};
use crate::codebase::postgres::{SqlColumnMetadata, SqlSchemaFileFacts, SqlTableSchemaEvent};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Tracked {
    pub(super) table: String,
    pub(super) column: String,
    pub(super) source: String,
    pub(super) function: Option<String>,
}

pub(super) type LiveColumns = BTreeMap<String, BTreeMap<String, SqlColumnMetadata>>;

pub(super) fn live_columns(schema: &[&SqlSchemaFileFacts]) -> LiveColumns {
    let mut live = LiveColumns::new();
    for file in schema {
        if !file.table_events_collected {
            for table in &file.tables {
                live.insert(table.table_name.clone(), column_map(&table.columns));
            }
        }
        for event in &file.table_events {
            match event {
                SqlTableSchemaEvent::Create { table, columns, .. } => {
                    live.insert(table.clone(), column_map(columns));
                }
                SqlTableSchemaEvent::AddColumn {
                    table,
                    column,
                    table_if_exists,
                    ..
                } => {
                    if *table_if_exists && !live.contains_key(table) {
                        continue;
                    }
                    live.entry(table.clone())
                        .or_default()
                        .insert(column.name.clone(), column.clone());
                }
                SqlTableSchemaEvent::Drop { table, .. } => {
                    live.remove(table);
                }
            }
        }
    }
    live
}

fn column_map(columns: &[SqlColumnMetadata]) -> BTreeMap<String, SqlColumnMetadata> {
    columns
        .iter()
        .map(|column| (column.name.clone(), column.clone()))
        .collect()
}

pub(super) fn tracked_columns(live: &LiveColumns, opts: &CompiledOptions) -> Vec<Tracked> {
    let mut tracked = Vec::new();
    for (name, columns) in live {
        let keys: Vec<_> = columns
            .iter()
            .filter(|(_, column)| column.is_primary_key)
            .map(|(name, _)| name)
            .collect();
        for (column_name, column) in columns {
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
            let [source] = column.generated_function_arg_columns.as_slice() else {
                continue;
            };
            if opts.require_primary_key && !(keys.len() == 1 && keys[0] == source) {
                continue;
            }
            tracked.push(Tracked {
                table: name.clone(),
                column: column_name.clone(),
                source: source.clone(),
                function: Some(function.to_string()),
            });
        }
    }
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
            function: None,
        });
    }
    tracked
}

pub(super) fn column_index(live: &LiveColumns) -> BTreeMap<String, BTreeSet<String>> {
    live.iter()
        .map(|(table, columns)| (table.clone(), columns.keys().cloned().collect()))
        .collect()
}

pub(super) fn stale_extras(
    live: &LiveColumns,
    extras: &[ExtraGeneratedColumn],
) -> Vec<RuleFinding> {
    extras
        .iter()
        .filter(|extra| {
            live.get(extra.table.trim())
                .and_then(|columns| columns.get(extra.column.trim()))
                .is_some_and(|column| column.is_generated)
        })
        .map(|extra| RuleFinding {
            source_offset: None,
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

#[cfg(test)]
mod tests;
