//! Narrow trait and data hooks inverting catalog dependencies from SQL analysis.

use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
    Truncate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogTriggerWrite {
    pub relation: String,
    pub event: TriggerEvent,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchPathResolution {
    Temporary,
    Physical(String),
    Unknown,
}

/// Catalog metadata needed when projecting temporary-relation lifetimes and
/// user-defined scalar functions onto prepared statement bounds.
pub trait CatalogLifecycleLookup {
    fn current_database(&self) -> Option<&str>;
    fn resolve_before_temp(&self, earlier: &[String], name: &str) -> SearchPathResolution;
    fn scalar_function_names(&self) -> Arc<BTreeSet<Vec<String>>>;
}
