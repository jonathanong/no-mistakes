use crate::codebase::ts_source::SourceStore;
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

mod build;
mod expressions;
mod findings;
mod function;
mod function_body;
mod function_comment;
mod function_quote;
mod model;
mod names;
mod order;
mod partition;
mod resolve;
mod snapshot;
#[cfg(test)]
mod tests;
mod trigger;

pub use expressions::{
    expression_matches, normalize_expression, order_prefix_matches, parse_postgres_expression,
};
pub use findings::{
    catalog_finding, require_catalog_path, AllowEntry, AllowList, CatalogObjectRef,
};
pub use model::{
    CatalogCheck, CatalogColumn, CatalogEnum, CatalogForeignKey, CatalogFunction, CatalogIndexInfo,
    CatalogIndexKey, CatalogTable, CatalogTrigger, CatalogUnique, CatalogView, GeneratedKind,
    PartitionKey, PartitionKeyElement, PartitionStrategy, RelationKind, TriggerEvent,
    TriggerTiming,
};
pub(crate) use order::{canonical_order_keys, order_by_ascending};
use snapshot::Snapshot;

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
    tables: BTreeMap<String, ArbiterTable>,
    model_tables: BTreeMap<String, CatalogTable>,
    functions: BTreeMap<String, CatalogFunction>,
    enums: BTreeMap<String, CatalogEnum>,
    views: BTreeMap<String, CatalogView>,
}
#[derive(Debug, Clone, Default)]
struct ArbiterTable {
    indexes: Vec<CanonicalIndex>,
    unique_constraints: BTreeMap<String, Vec<String>>,
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
            bail!(
                "schemaCatalogPath {} must be a PostgreSQL schema snapshot with formatVersion 2",
                path.display()
            );
        }
        Self::from_snapshot(&path.display().to_string(), snapshot)
    }

    fn from_snapshot(path: &str, snapshot: Snapshot) -> Result<Self> {
        build::from_snapshot(path, snapshot)
    }

    pub fn tables(&self) -> impl Iterator<Item = &CatalogTable> {
        self.model_tables.values()
    }

    pub fn table(&self, name: &str) -> Option<&CatalogTable> {
        self.model_tables.get(name)
    }

    pub fn functions(&self) -> impl Iterator<Item = &CatalogFunction> {
        self.functions.values()
    }

    pub fn enums(&self) -> impl Iterator<Item = &CatalogEnum> {
        self.enums.values()
    }

    pub fn views(&self) -> impl Iterator<Item = &CatalogView> {
        self.views.values()
    }
}

pub(crate) fn normalize_catalog_path(raw_path: &str) -> Result<PathBuf> {
    let path = Path::new(raw_path);
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("schemaCatalogPath must be a non-empty repository-relative path");
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        bail!("schemaCatalogPath must be a non-empty repository-relative path");
    }
    Ok(normalized)
}

fn catalog_path(root: &Path, raw_path: &str) -> Result<PathBuf> {
    Ok(root.join(normalize_catalog_path(raw_path)?))
}
