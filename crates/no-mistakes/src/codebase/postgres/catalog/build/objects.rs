use super::super::model::{
    CatalogCheck, CatalogForeignKey, CatalogIndexInfo, CatalogIndexKey, CatalogTrigger,
    CatalogUnique,
};
use super::super::partition::{parse_partition_key, PartitionParseError};
use super::super::snapshot::SnapshotTable;
use super::super::trigger::parse_trigger;
use anyhow::{bail, Result};

pub(super) fn foreign_keys(table: &SnapshotTable) -> Vec<CatalogForeignKey> {
    table
        .foreign_keys
        .iter()
        .map(|(name, key)| CatalogForeignKey {
            name: name.clone(),
            columns: key.columns.clone(),
            referenced_table: key.referenced_table.clone(),
            referenced_columns: key.referenced_columns.clone(),
            on_delete: key.on_delete.to_ascii_lowercase(),
            on_update: key.on_update.to_ascii_lowercase(),
            validated: key.validated,
        })
        .collect()
}

pub(super) fn checks(table: &SnapshotTable) -> Vec<CatalogCheck> {
    table
        .check_constraints
        .iter()
        .map(|(name, check)| CatalogCheck {
            name: name.clone(),
            definition: check.definition.clone(),
        })
        .collect()
}

pub(super) fn uniques(table: &SnapshotTable) -> Vec<CatalogUnique> {
    table
        .unique_constraints
        .iter()
        .map(|(name, constraint)| CatalogUnique {
            name: name.clone(),
            columns: constraint.columns.clone(),
        })
        .collect()
}

pub(super) fn indexes(table: &SnapshotTable) -> Vec<CatalogIndexInfo> {
    table
        .indexes
        .iter()
        .map(|(name, index)| CatalogIndexInfo {
            name: name.clone(),
            unique: index.unique,
            primary: index.primary,
            constraint_backed: index.constraint_backed,
            access_method: index.access_method.clone(),
            keys: index
                .keys
                .iter()
                .map(|key| CatalogIndexKey {
                    column: key.column.clone(),
                    expression: key.expression.clone(),
                })
                .collect(),
            predicate: index.predicate.clone(),
            definition: index.definition.clone(),
        })
        .collect()
}

pub(super) fn triggers(
    path: &str,
    table: &str,
    snapshot: &SnapshotTable,
) -> Result<Vec<CatalogTrigger>> {
    snapshot
        .triggers
        .iter()
        .map(|(name, trigger)| {
            let parsed = parse_trigger(&trigger.definition).map_err(|_| {
                anyhow::anyhow!(
                    "schemaCatalogPath {path}: table {table} trigger {name} has an unreadable definition"
                )
            })?;
            Ok(CatalogTrigger {
                name: name.clone(),
                timing: parsed.timing,
                events: parsed.events,
                update_columns: parsed.update_columns,
                for_each_row: parsed.for_each_row,
                function: parsed.function,
                arguments: parsed.arguments,
                when: parsed.when,
                definition: trigger.definition.clone(),
            })
        })
        .collect()
}

pub(super) fn partition_key(
    path: &str,
    table: &str,
    snapshot: &SnapshotTable,
) -> Result<Option<super::super::model::PartitionKey>> {
    let Some(partition) = &snapshot.physical_partition else {
        return Ok(None);
    };
    match parse_partition_key(&partition.key) {
        Ok(key) => Ok(Some(key)),
        Err(PartitionParseError::Strategy(strategy)) => bail!(
            "schemaCatalogPath {path}: table {table} has unsupported partition strategy {strategy}"
        ),
        Err(PartitionParseError::Syntax) => {
            bail!("schemaCatalogPath {path}: table {table} has unreadable partition key")
        }
    }
}
