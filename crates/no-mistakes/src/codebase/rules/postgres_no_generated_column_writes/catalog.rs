use super::ExtraGeneratedColumn;
use crate::codebase::postgres::dml::{GeneratedTable, GeneratedTableColumns};

mod configured;
mod live;
pub(super) use live::{live_tables, LiveTables};

pub(super) fn catalog_from_tables(
    tables: &LiveTables<'_>,
    extra: &[ExtraGeneratedColumn],
) -> GeneratedTableColumns {
    let mut catalog = empty_catalog_for_tables(tables);
    for (key, table) in tables {
        let generated = table
            .columns
            .iter()
            .filter(|column| column.generated)
            .map(|column| column.name.to_ascii_lowercase())
            .collect::<std::collections::BTreeSet<_>>();
        if generated.is_empty() {
            continue;
        }
        catalog.insert_table_with_key(
            key,
            GeneratedTable {
                name: catalog.display_name(key, table.name),
                generated,
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
    for extra in extra {
        if extra.table.is_empty() || extra.column.is_empty() {
            continue;
        }
        let Some(key) = configured::relation(tables, &extra.table) else {
            continue;
        };
        let base = crate::codebase::postgres::idents::relation_suffix_name(&key);
        catalog.register_relation(&key, &base);
        catalog.insert_table_with_key(
            &key,
            GeneratedTable {
                name: catalog.display_name(&key, &base),
                generated: [extra.column.to_ascii_lowercase()].into_iter().collect(),
                column_order: None,
            },
        );
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
    extras
        .iter()
        .filter(|extra| !extra.table.is_empty() && !extra.column.is_empty())
        .filter(|extra| {
            configured::relation(tables, &extra.table)
                .and_then(|key| tables.get(&key))
                .is_some_and(|table| table.columns.iter().any(|column| {
                    column.generated && column.name.eq_ignore_ascii_case(&extra.column)
                }))
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

#[cfg(test)]
mod tests;

fn empty_catalog_for_tables(tables: &LiveTables<'_>) -> GeneratedTableColumns {
    let mut catalog = GeneratedTableColumns::default();
    for (key, table) in tables {
        catalog.register_relation(key, table.name);
        if table.temporary {
            catalog.prefer_relation(key, table.name);
        }
    }
    catalog
}
