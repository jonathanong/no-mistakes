use crate::codebase::ts_source::SourceStore;
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

mod expressions;
mod resolve;
#[cfg(test)]
mod tests;

pub use expressions::{
    expression_matches, normalize_expression, order_prefix_matches, parse_postgres_expression,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalOrderKey {
    pub expression: String,
    pub ascending: bool,
    pub nulls_first: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalIndex {
    pub name: String,
    pub constraint_backed: bool,
    pub keys: Vec<CanonicalOrderKey>,
    pub predicate: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedArbiter {
    Exact(CanonicalIndex),
    Ambiguous,
    Unresolved,
}
#[derive(Debug, Clone, Default)]
pub struct SchemaCatalog {
    tables: BTreeMap<String, CatalogTable>,
}
#[derive(Debug, Clone, Default)]
struct CatalogTable {
    indexes: Vec<CanonicalIndex>,
    unique_constraints: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    format_version: u32,
    #[serde(default)]
    tables: BTreeMap<String, SnapshotTable>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SnapshotTable {
    #[serde(default)]
    indexes: BTreeMap<String, SnapshotIndex>,
    #[serde(default)]
    unique_constraints: BTreeMap<String, SnapshotKeyConstraint>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SnapshotKeyConstraint {
    #[serde(default)]
    columns: Vec<String>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SnapshotIndex {
    #[serde(default)]
    access_method: String,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    constraint_backed: bool,
    #[serde(default)]
    valid: bool,
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    keys: Vec<SnapshotIndexKey>,
    predicate: Option<String>,
}
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SnapshotIndexKey {
    #[serde(default)]
    expression: String,
    #[serde(default)]
    descending: bool,
    #[serde(default)]
    nulls_first: bool,
}

impl SchemaCatalog {
    pub fn load(root: &Path, raw_path: &str, sources: &SourceStore) -> Result<Self> {
        let path = catalog_path(root, raw_path)?;
        let source = sources.read_path(&path).map_err(|error| {
            anyhow::anyhow!(
                "failed to read schemaCatalogPath {}: {error}",
                path.display()
            )
        })?;
        let snapshot: Snapshot = serde_json::from_str(&source)
            .with_context(|| format!("schemaCatalogPath {} is not valid JSON", path.display()))?;
        if snapshot.format_version != 2 {
            bail!("schemaCatalogPath {} must be a Vouchington PostgreSQL schema snapshot formatVersion 2", path.display());
        }
        Ok(Self::from_snapshot(snapshot))
    }

    fn from_snapshot(snapshot: Snapshot) -> Self {
        let tables = snapshot
            .tables
            .into_iter()
            .map(|(name, table)| {
                let indexes = table
                    .indexes
                    .into_iter()
                    .filter_map(|(name, index)| {
                        ((index.unique || index.primary)
                            && index.valid
                            && index.ready
                            && index.access_method.eq_ignore_ascii_case("btree")
                            && !index.keys.is_empty())
                        .then(|| CanonicalIndex {
                            name,
                            constraint_backed: index.constraint_backed,
                            predicate: index.predicate.map(|value| normalize_expression(&value)),
                            keys: index
                                .keys
                                .into_iter()
                                .filter(|key| !key.expression.is_empty())
                                .map(|key| CanonicalOrderKey {
                                    expression: key.expression,
                                    ascending: !key.descending,
                                    nulls_first: key.nulls_first,
                                })
                                .collect(),
                        })
                    })
                    .filter(|index| !index.keys.is_empty())
                    .collect();
                let unique_constraints = table
                    .unique_constraints
                    .into_iter()
                    .map(|(name, constraint)| {
                        (
                            normalize_identifier(&name),
                            constraint
                                .columns
                                .into_iter()
                                .map(|column| normalize_expression(&column))
                                .collect(),
                        )
                    })
                    .collect();
                (
                    normalize_identifier(&name),
                    CatalogTable {
                        indexes,
                        unique_constraints,
                    },
                )
            })
            .collect();
        Self { tables }
    }
}

fn normalize_identifier(identifier: &str) -> String {
    identifier
        .rsplit('.')
        .next()
        .unwrap_or(identifier)
        .trim_matches('"')
        .to_ascii_lowercase()
}
fn catalog_path(root: &Path, raw_path: &str) -> Result<PathBuf> {
    let path = Path::new(raw_path);
    if raw_path.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        bail!("schemaCatalogPath must be a non-empty repository-relative path");
    }
    Ok(root.join(path))
}
