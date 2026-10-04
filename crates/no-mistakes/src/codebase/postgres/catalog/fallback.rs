//! Construct fallback indexes only when exact catalog lookup is insufficient.
use super::{names, CatalogTable};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct RelationFallback {
    pub(super) unique: BTreeMap<String, Option<String>>,
    pub(super) bare_keys: BTreeMap<String, String>,
}

pub(super) fn build(tables: &BTreeMap<String, CatalogTable>) -> RelationFallback {
    let mut unique = BTreeMap::new();
    let mut bare_keys = BTreeMap::new();
    for key in tables.keys() {
        let (qualifier, bare) = names::split_key(key);
        unique
            .entry(bare.clone())
            .and_modify(|value| *value = None)
            .or_insert_with(|| Some(key.clone()));
        if qualifier.is_none() {
            bare_keys.insert(bare, key.clone());
        }
    }
    RelationFallback { unique, bare_keys }
}
