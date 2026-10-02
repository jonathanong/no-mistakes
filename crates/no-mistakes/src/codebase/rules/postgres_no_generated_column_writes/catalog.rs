use super::ExtraGeneratedColumn;
use crate::codebase::postgres::dml::{GeneratedTable, GeneratedTableColumns};

mod live;
pub(super) use live::{live_tables, LiveTables};

pub(super) fn catalog_from_tables(
    tables: &LiveTables<'_>,
    extra: &[ExtraGeneratedColumn],
) -> GeneratedTableColumns {
    let mut catalog = GeneratedTableColumns::default();
    for table in tables.values() {
        let generated = table
            .columns
            .iter()
            .filter(|column| column.generated)
            .map(|column| column.name.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if generated.is_empty() {
            continue;
        }
        catalog.insert_table(GeneratedTable {
            name: table.name.to_string(),
            generated,
            column_order: table.complete.then(|| {
                table
                    .columns
                    .iter()
                    .map(|column| column.name.to_ascii_lowercase())
                    .collect()
            }),
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
pub(super) use trigger::trigger_catalog_from_tables;

pub(super) fn stale_trigger_findings(
    tables: &LiveTables<'_>,
    columns: &[String],
) -> Vec<crate::codebase::rules::RuleFinding> {
    columns
        .iter()
        .filter(|column| !schema_has_column(tables, column))
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

fn schema_has_column(tables: &LiveTables<'_>, name: &str) -> bool {
    tables.values().any(|table| {
        table
            .columns
            .iter()
            .any(|column| column.name.eq_ignore_ascii_case(name))
    })
}

pub(super) fn stale_extra_findings_from_tables(
    tables: &LiveTables<'_>,
    extras: &[ExtraGeneratedColumn],
) -> Vec<crate::codebase::rules::RuleFinding> {
    let in_schema = schema_generated(tables);
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

fn schema_generated(tables: &LiveTables<'_>) -> std::collections::BTreeSet<(String, String)> {
    tables
        .iter()
        .flat_map(|(key, table)| {
            table
                .columns
                .iter()
                .filter(|column| column.generated)
                .map(move |column| (key.clone(), column.name.to_ascii_lowercase()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
