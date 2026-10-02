use crate::codebase::ts_source::SourceStore;
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

mod build;
mod expressions;
mod findings;
mod function;
mod function_body;
mod function_clauses;
mod function_comment;
mod function_escape;
mod function_outputs;
mod function_quote;
mod locations;
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
    column_lines: BTreeMap<(String, String), usize>,
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
        let parsed = jsonc_parser::parse_to_ast(
            &source,
            &jsonc_parser::CollectOptions::default(),
            &jsonc_parser::ParseOptions {
                allow_comments: true,
                allow_loose_object_property_names: false,
                allow_trailing_commas: false,
                allow_missing_commas: false,
                allow_single_quoted_strings: false,
                allow_hexadecimal_numbers: false,
                allow_unary_plus_numbers: false,
            },
        )
        .with_context(|| format!("schemaCatalogPath {} is not valid JSONC", path.display()))?;
        let column_lines = parsed
            .value
            .as_ref()
            .map(|value| locations::column_lines(value, &source))
            .unwrap_or_default();
        let snapshot: Snapshot = serde_json::from_value(
            parsed
                .value
                .ok_or_else(|| anyhow::anyhow!("schemaCatalogPath {} is empty", path.display()))?
                .into(),
        )
        .with_context(|| format!("schemaCatalogPath {} has an invalid schema", path.display()))?;
        if snapshot.format_version != 2 {
            bail!(
                "schemaCatalogPath {} must be a PostgreSQL schema snapshot with formatVersion 2",
                path.display()
            );
        }
        let mut catalog = Self::from_snapshot(&path.display().to_string(), snapshot)?;
        catalog.set_column_lines(column_lines);
        Ok(catalog)
    }

    fn from_snapshot(path: &str, snapshot: Snapshot) -> Result<Self> {
        build::from_snapshot(path, snapshot)
    }

    pub fn tables(&self) -> impl Iterator<Item = &CatalogTable> {
        self.model_tables.values()
    }

    /// Line of the column declaration in the JSONC schema snapshot.
    pub(crate) fn column_line(&self, table: &str, column: &str) -> usize {
        self.column_lines
            .get(&(table.to_string(), column.to_string()))
            .copied()
            .unwrap_or(1)
    }

    pub(crate) fn set_column_lines(&mut self, lines: BTreeMap<(String, String), usize>) {
        self.column_lines = lines;
    }

    pub fn table(&self, name: &str) -> Option<&CatalogTable> {
        self.model_tables.get(name)
    }

    pub fn relation(&self, name: &str) -> Option<&CatalogTable> {
        let normalized = names::normalize_table_name(name);
        if let Some(table) = self.model_tables.get(&normalized) {
            return Some(table);
        }
        let tail = normalized.rsplit('.').next().unwrap_or(&normalized);
        let mut matches = self
            .model_tables
            .iter()
            .filter(|(key, _)| key.rsplit('.').next().unwrap_or(key) == tail);
        let (_, table) = matches.next()?;
        matches.next().is_none().then_some(table)
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
