//! Request-local SQL relation identities and dependency closure.
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(super) struct State {
    pub relations: BTreeMap<String, BTreeSet<String>>,
    pub on_commit_drop: BTreeSet<String>,
    pub temp_first: bool,
    pub local_path: Option<bool>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            relations: BTreeMap::new(),
            on_commit_drop: BTreeSet::new(),
            temp_first: true,
            local_path: None,
        }
    }
}
pub(super) fn key(name: &str) -> String {
    decoded_parts(name).last().cloned().unwrap_or_default()
}
impl State {
    pub fn contains(&self, name: &str) -> bool {
        let parts = decoded_parts(name);
        (parts.len() == 1 && self.temp_first || parts.len() == 2 && parts[0] == "pg_temp")
            && self.relations.contains_key(&key(name))
    }
    pub fn dependencies(&self, query: &SqlBoundQuery, out: &mut BTreeSet<String>) {
        for item in &query.items {
            match &item.kind {
                SqlBoundItemKind::Table(name) if self.contains(name) => {
                    out.insert(key(name));
                }
                SqlBoundItemKind::Query(query) => self.dependencies(query, out),
                _ => {}
            }
            for pin in &item.pins {
                if let SqlPinSource::Query(query) = &pin.source {
                    self.dependencies(query, out);
                }
            }
        }
    }
    pub fn drop(&mut self, name: &str, cascade: bool) {
        if !self.contains(name) {
            return;
        }
        self.remove(BTreeSet::from([key(name)]), cascade);
    }
    pub fn commit(&mut self) {
        let removed = std::mem::take(&mut self.on_commit_drop);
        self.remove(removed, true);
    }
    fn remove(&mut self, mut removed: BTreeSet<String>, cascade: bool) {
        if cascade {
            loop {
                let before = removed.len();
                for (name, dependencies) in &self.relations {
                    if dependencies
                        .iter()
                        .any(|dependency| removed.contains(dependency))
                    {
                        removed.insert(name.clone());
                    }
                }
                if before == removed.len() {
                    break;
                }
            }
        }
        self.relations.retain(|name, _| !removed.contains(name));
        self.on_commit_drop.retain(|name| !removed.contains(name));
    }
    pub fn rename(&mut self, old: &str, new: &str) {
        if !self.contains(old) {
            return;
        }
        let old = key(old);
        let new = key(new);
        if self.on_commit_drop.remove(&old) {
            self.on_commit_drop.insert(new.clone());
        }
        let dependencies = self.relations.remove(&old).unwrap_or_default();
        self.relations.insert(new.clone(), dependencies);
        for dependencies in self.relations.values_mut() {
            if dependencies.remove(&old) {
                dependencies.insert(new.clone());
            }
        }
    }
}
