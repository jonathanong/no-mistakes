use super::super::model::{CatalogColumn, GeneratedKind, RelationKind};
use super::super::snapshot::SnapshotColumn;
use anyhow::{bail, Result};
use std::collections::BTreeMap;

pub(super) fn relation_kind(path: &str, table: &str, raw: &str) -> Result<RelationKind> {
    match raw.to_ascii_lowercase().as_str() {
        "" | "table" => Ok(RelationKind::Table),
        "partitioned table" => Ok(RelationKind::PartitionedTable),
        other => {
            bail!("schemaCatalogPath {path}: table {table} has unsupported relation kind {other}")
        }
    }
}

pub(super) fn columns(
    path: &str,
    table: &str,
    columns: &BTreeMap<String, SnapshotColumn>,
) -> Result<Vec<CatalogColumn>> {
    let mut columns = columns
        .iter()
        .map(|(name, column)| {
            Ok(CatalogColumn {
                name: name.clone(),
                data_type: column.data_type.clone(),
                nullable: column.nullable,
                default_expression: column.default_expression.clone(),
                generated: generated_kind(path, table, name, column.generated.as_deref())?,
                generated_expression: column.generated_expression.clone(),
                identity: column.identity.clone(),
                comment: column.comment.clone(),
                ordinal_position: column.ordinal_position,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    columns.sort_by(|left, right| {
        left.ordinal_position
            .cmp(&right.ordinal_position)
            .then(left.name.cmp(&right.name))
    });
    Ok(columns)
}

fn generated_kind(
    path: &str,
    table: &str,
    column: &str,
    raw: Option<&str>,
) -> Result<Option<GeneratedKind>> {
    match raw.map(|value| value.to_ascii_lowercase()).as_deref() {
        None => Ok(None),
        Some("stored") => Ok(Some(GeneratedKind::Stored)),
        Some("virtual") => Ok(Some(GeneratedKind::Virtual)),
        Some(other) => bail!(
            "schemaCatalogPath {path}: table {table} column {column} has unsupported generated kind {other}"
        ),
    }
}
