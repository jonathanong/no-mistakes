mod arbiter;
mod columns;
mod objects;

use super::function::function_from_definition;
use super::model::{CatalogEnum, CatalogFunction, CatalogTable, CatalogView};
use super::names::normalize_table_name;
use super::snapshot::{Snapshot, SnapshotTable};
use super::SchemaCatalog;
use anyhow::Result;
use std::collections::BTreeMap;

pub(super) fn from_snapshot(path: &str, snapshot: Snapshot) -> Result<SchemaCatalog> {
    let mut tables = BTreeMap::new();
    let mut model_tables = BTreeMap::new();
    for (name, table) in snapshot.tables {
        let model = model_table(path, &name, &table)?;
        tables.insert(normalize_table_name(&name), arbiter::arbiter_table(&table));
        model_tables.insert(normalize_table_name(&name), model);
    }
    Ok(SchemaCatalog {
        coverage: snapshot.coverage,
        schema: snapshot.schema,
        tables,
        model_tables,
        functions: functions(snapshot.functions),
        enums: enums(snapshot.enums),
        views: views(snapshot.views),
        column_lines: BTreeMap::new(),
    })
}

fn functions(
    functions: BTreeMap<String, super::snapshot::SnapshotFunction>,
) -> BTreeMap<String, CatalogFunction> {
    functions
        .into_iter()
        .map(|(key, function)| {
            (
                key.clone(),
                function_from_definition(&key, &function.definition),
            )
        })
        .collect()
}

fn enums(enums: BTreeMap<String, super::snapshot::SnapshotEnum>) -> BTreeMap<String, CatalogEnum> {
    enums
        .into_iter()
        .map(|(name, enum_type)| {
            (
                name.clone(),
                CatalogEnum {
                    name,
                    values: enum_type.values,
                },
            )
        })
        .collect()
}

fn views(views: BTreeMap<String, super::snapshot::SnapshotView>) -> BTreeMap<String, CatalogView> {
    views
        .into_iter()
        .map(|(name, view)| {
            (
                name.clone(),
                CatalogView {
                    name,
                    materialized: view.materialized,
                    definition: view.definition,
                    comment: view.comment,
                },
            )
        })
        .collect()
}

fn model_table(path: &str, name: &str, table: &SnapshotTable) -> Result<CatalogTable> {
    Ok(CatalogTable {
        name: name.to_string(),
        relation_kind: columns::relation_kind(path, name, &table.relation_kind)?,
        comment: table.comment.clone(),
        columns: columns::columns(path, name, &table.columns)?,
        primary_key: table.primary_key.as_ref().map(|key| key.columns.clone()),
        foreign_keys: objects::foreign_keys(table),
        check_constraints: objects::checks(table),
        unique_constraints: objects::uniques(table),
        indexes: objects::indexes(table),
        triggers: objects::triggers(path, name, table)?,
        partition_key: objects::partition_key(path, name, table)?,
    })
}
