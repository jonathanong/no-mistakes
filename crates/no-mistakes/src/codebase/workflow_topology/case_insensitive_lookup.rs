//! Case-insensitive name resolution, ported from `case-insensitive-lookup.mts`.
//!
//! GitHub Actions job keys, `workflow_call` input/secret/output names, and
//! `workflow_run` source names are all matched case-insensitively. A name
//! matching more than one entry after lowercasing is ambiguous and treated
//! conservatively (skipped, not guessed) by every caller.
//!
//! Grouping uses a plain `HashMap`, not an order-preserving map: none of
//! this module's callers observe group iteration order in their final
//! output — diagnostics are always globally re-sorted by content in
//! `topology_graph::sort_topology` regardless of the order they were
//! pushed in.

use std::collections::HashMap;

pub enum Resolution<'a, T> {
    Missing,
    Ambiguous,
    Resolved { key: &'a str, value: &'a T },
}

pub struct CaseInsensitiveLookup<T> {
    groups: HashMap<String, Vec<(String, T)>>,
}

impl<T> CaseInsensitiveLookup<T> {
    pub fn new(entries: impl IntoIterator<Item = (String, T)>) -> Self {
        let mut groups: HashMap<String, Vec<(String, T)>> = HashMap::new();
        for (key, value) in entries {
            groups
                .entry(key.to_lowercase())
                .or_default()
                .push((key, value));
        }
        Self { groups }
    }

    pub fn resolve(&self, key: &str) -> Resolution<'_, T> {
        match self.groups.get(&key.to_lowercase()).map(Vec::as_slice) {
            None | Some([]) => Resolution::Missing,
            Some([(key, value)]) => Resolution::Resolved { key, value },
            Some(_) => Resolution::Ambiguous,
        }
    }

    /// Entries whose lowercased key matched exactly one original entry.
    pub fn unique_entries(&self) -> impl Iterator<Item = (&str, &T)> {
        self.groups
            .values()
            .filter(|matches| matches.len() == 1)
            .map(|matches| (matches[0].0.as_str(), &matches[0].1))
    }
}
