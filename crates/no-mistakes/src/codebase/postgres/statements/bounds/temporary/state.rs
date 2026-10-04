//! Request-local SQL relation identities and dependency closure.
mod names;
mod ownership;
mod rename;
mod schema;
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::statements::{
    SqlBoundItemKind, SqlBoundQuery, SqlPinSource, SqlPossibleTemporary,
};
use names::names_match;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Dependency {
    Temporary(String),
    Physical(Vec<String>),
}

#[derive(Clone)]
pub(super) struct State {
    pub relations: BTreeMap<String, BTreeSet<Dependency>>,
    // A partition is owned by its parent even for DROP without CASCADE.
    pub partitions: BTreeMap<String, String>,
    pub partitioned: BTreeSet<String>,
    pub on_commit_drop: BTreeSet<String>,
    pub temp_first: bool,
    pub earlier_schemas: Option<Vec<String>>,
    pub local_path: Option<(bool, Option<Vec<String>>)>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            relations: BTreeMap::new(),
            partitions: BTreeMap::new(),
            partitioned: BTreeSet::new(),
            on_commit_drop: BTreeSet::new(),
            temp_first: true,
            earlier_schemas: None,
            local_path: None,
        }
    }
}
pub(super) fn key(name: &str) -> String {
    decoded_parts(name).last().cloned().unwrap_or_default()
}
impl State {
    pub fn include_dependencies(&self, name: &str, out: &mut BTreeSet<Dependency>) {
        if self.contains(name) {
            out.insert(Dependency::Temporary(key(name)));
        } else {
            out.insert(Dependency::Physical(decoded_parts(name)));
            if self.possible_temporary(name).is_some() {
                // An unknown earlier schema can make the source physical or temporary.
                // Keep both possibilities so a later CASCADE cannot leave a stale view.
                out.insert(Dependency::Temporary(key(name)));
            }
        }
    }

    pub fn possible_temporary(&self, name: &str) -> Option<SqlPossibleTemporary> {
        let parts = decoded_parts(name);
        (parts.len() == 1 && !self.temp_first && self.relations.contains_key(&parts[0]))
            .then(|| self.earlier_schemas.clone())
            .flatten()
            .map(|earlier_schemas| SqlPossibleTemporary {
                database_qualifier: None,
                earlier_schemas,
            })
    }
    pub fn contains(&self, name: &str) -> bool {
        let parts = decoded_parts(name);
        (parts.len() == 1 && self.temp_first || parts.len() == 2 && parts[0] == "pg_temp")
            && self.relations.contains_key(&key(name))
    }
    pub fn dependencies(&self, query: &SqlBoundQuery, out: &mut BTreeSet<Dependency>) {
        for item in &query.items {
            match &item.kind {
                SqlBoundItemKind::Table(name) => {
                    self.include_dependencies(name, out);
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
        let mut removed = BTreeSet::new();
        let definite_temporary = self.contains(name);
        // An unqualified name behind unknown earlier schemas may denote this temporary
        // relation. Retire it conservatively: retaining it after a real DROP can hide a
        // catalog read, while retiring it after a different DROP can only over-report.
        if definite_temporary || self.possible_temporary(name).is_some() {
            removed.insert(key(name));
        }
        if cascade && !definite_temporary {
            let parts = decoded_parts(name);
            removed.extend(
                self.relations
                    .iter()
                    .filter(|(_, dependencies)| {
                        dependencies.iter().any(|dependency| {
                            matches!(dependency, Dependency::Physical(source) if names_match(source, &parts))
                        })
                    })
                    .map(|(name, _)| name.clone()),
            );
        }
        self.remove(removed, cascade);
    }
    pub fn commit(&mut self) {
        let removed = std::mem::take(&mut self.on_commit_drop);
        self.remove(removed, true);
    }
    fn remove(&mut self, mut removed: BTreeSet<String>, cascade: bool) {
        loop {
            let before = removed.len();
            for (child, parent) in &self.partitions {
                if removed.contains(parent) {
                    removed.insert(child.clone());
                }
            }
            if cascade {
                for (name, dependencies) in &self.relations {
                    if dependencies.iter().any(|dependency| {
                        matches!(dependency, Dependency::Temporary(parent) if removed.contains(parent))
                    }) {
                        removed.insert(name.clone());
                    }
                }
            }
            if before == removed.len() {
                break;
            }
        }
        self.relations.retain(|name, _| !removed.contains(name));
        self.partitions.retain(|child, _| !removed.contains(child));
        self.partitioned.retain(|name| !removed.contains(name));
        self.on_commit_drop.retain(|name| !removed.contains(name));
    }
}
