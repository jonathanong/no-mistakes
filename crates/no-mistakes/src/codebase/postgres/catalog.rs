use crate::codebase::ts_source::SourceStore;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

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
mod load;
mod locations;
mod model;
mod names;
mod order;
mod partition;
mod paths;
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
    CatalogCheck, CatalogColumn, CatalogCoverage, CatalogEnum, CatalogForeignKey, CatalogFunction,
    CatalogIndexInfo, CatalogIndexKey, CatalogTable, CatalogTrigger, CatalogUnique, CatalogView,
    GeneratedKind, PartitionKey, PartitionKeyElement, PartitionStrategy, RelationKind,
    TriggerEvent, TriggerTiming,
};
pub(crate) use order::{canonical_order_keys, order_by_ascending};
use paths::catalog_path;
pub(crate) use paths::normalize_catalog_path;
use snapshot::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalOrderKey {
    pub expression: String,
    pub ascending: bool,
    pub nulls_first: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalIndex {
    pub ordering_supported: bool,
    pub immediate: bool,
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
    coverage: CatalogCoverage,
    schema: Option<String>,
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
    pub fn coverage(&self) -> CatalogCoverage {
        self.coverage
    }

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
        let value = parsed
            .value
            .ok_or_else(|| anyhow::anyhow!("schemaCatalogPath {} is empty", path.display()))?;
        let snapshot = load::parse_snapshot(&path.display().to_string(), value.into())?;
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

    /// Tables are indexed by normalized name, so a quoted key such as `"Order Items"` is found
    /// by its quoted or unquoted spelling, the way it round-trips from the generator.
    pub fn table(&self, name: &str) -> Option<&CatalogTable> {
        self.model_tables.get(&names::normalize_table_name(name))
    }

    /// The table a SQL name refers to: its exact key, or the one table whose bare name it is.
    /// A name that spells a schema reaches a bare-keyed table only when it is this catalog's
    /// schema; `audit.accounts` is not the `accounts` of the catalog for `public`.
    pub fn relation(&self, name: &str) -> Option<&CatalogTable> {
        let normalized = names::normalize_table_name(name);
        if let Some(table) = self.model_tables.get(&normalized) {
            return Some(table);
        }
        let tail = normalized.rsplit('.').next().unwrap_or(&normalized);
        let qualified = tail.len() < normalized.len();
        if qualified {
            let qualifier = &normalized[..normalized.len() - tail.len() - 1];
            let own = self.schema.as_deref().map(names::normalize_table_name);
            if own.is_some_and(|own| own != qualifier) {
                return None;
            }
        }
        // A qualified name that is not an exact key can only mean a bare-keyed table.
        let mut matches = self.model_tables.iter().filter(|(key, _)| {
            key.rsplit('.').next().unwrap_or(key) == tail && !(qualified && key.contains('.'))
        });
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

pub(crate) use names::normalize_table_name;
