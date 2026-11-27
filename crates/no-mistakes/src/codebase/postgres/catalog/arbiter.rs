use super::BoundedUniqueIndex;
pub use no_mistakes_postgres_sql::order::CanonicalOrderKey;
use std::collections::BTreeMap;

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
pub(super) struct ArbiterTable {
    pub(super) indexes: Vec<CanonicalIndex>,
    pub(super) bounded_indexes: Vec<BoundedUniqueIndex>,
    pub(super) unique_constraints: BTreeMap<String, Vec<String>>,
}
