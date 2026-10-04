//! Request-local SQL relation identities and dependency closure.
mod names;
mod ownership;
mod path;
mod physical;
mod rename;
mod schema;
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::{SchemaCatalog, SearchPathResolution};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Dependency {
    Temporary(String),
    ConditionalTemporary(String, String),
    // A bare read behind an unknown earlier search-path schema is not definite.
    PossibleTemporary(String),
    Physical(Vec<String>),
}

#[derive(Clone)]
pub(super) struct State {
    pub relations: BTreeMap<String, BTreeSet<Dependency>>,
    // Permanent-only views are dependency nodes, never temporary identities.
    pub physical_views: BTreeMap<Vec<String>, BTreeSet<Dependency>>,
    // A partition is owned by its parent even for DROP without CASCADE.
    pub partitions: BTreeMap<String, String>,
    pub partitioned: BTreeSet<String>,
    pub databases: BTreeMap<String, String>,
    /// Catalog-unknown DDL may have removed a temporary relation.
    pub uncertain_relations: BTreeSet<String>,
    /// A proven physical shadow removed during this SQL source's lifetime.
    pub removed_shadows: BTreeSet<(String, String)>,
    pub on_commit_drop: BTreeSet<String>,
    pub temp_first: bool,
    pub earlier_schemas: Option<Vec<String>>,
    pub search_path: Option<Vec<String>>,
    pub local_path: Option<path::Snapshot>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            relations: BTreeMap::new(),
            physical_views: BTreeMap::new(),
            partitions: BTreeMap::new(),
            partitioned: BTreeSet::new(),
            databases: BTreeMap::new(),
            uncertain_relations: BTreeSet::new(),
            removed_shadows: BTreeSet::new(),
            on_commit_drop: BTreeSet::new(),
            temp_first: true,
            earlier_schemas: None,
            search_path: None,
            local_path: None,
        }
    }
}
pub(super) fn key(name: &str) -> String {
    decoded_parts(name).last().cloned().unwrap_or_default()
}
impl State {
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
                if let SqlPinSource::Query(query) | SqlPinSource::ReadQuery(query) = &pin.source {
                    self.dependencies(query, out);
                }
            }
        }
    }
    pub fn drop(&mut self, name: &str, cascade: bool, catalog: Option<&SchemaCatalog>) {
        if let (Some(catalog), Some(candidate)) = (catalog, self.possible_temporary(name)) {
            let relation = key(name);
            if candidate.uncertain_lifetime {
                return;
            }
            if let Some(database) = candidate.database_qualifier.as_deref() {
                match catalog.current_database() {
                    Some(current) if current != database => return,
                    None => {
                        self.uncertain_relations.insert(relation);
                        return;
                    }
                    Some(_) => {}
                }
            }
            match catalog.resolve_before_temp(&candidate.earlier_schemas, name) {
                SearchPathResolution::Physical(schema) => {
                    let target = BTreeSet::from([vec![schema.clone(), relation.clone()]]);
                    if self.drop_physical(target, cascade) {
                        self.removed_shadows.insert((schema, relation));
                    }
                    return;
                }
                SearchPathResolution::Unknown => {
                    self.uncertain_relations.insert(relation);
                    return;
                }
                SearchPathResolution::Temporary => {
                    self.remove(BTreeSet::from([relation]), cascade);
                    return;
                }
            }
        }
        let mut removed = BTreeSet::new();
        let definite_temporary = self.matches_identity(name);
        // An unqualified name behind unknown earlier schemas may denote this temporary
        // relation. Retire it conservatively: retaining it after a real DROP can hide a
        // catalog read, while retiring it after a different DROP can only over-report.
        if definite_temporary || self.possible_temporary(name).is_some() {
            removed.insert(key(name));
        }
        if !definite_temporary {
            self.drop_physical(self.ddl_names(name), cascade);
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
                        matches!(dependency, Dependency::Temporary(parent) | Dependency::ConditionalTemporary(parent, _) | Dependency::PossibleTemporary(parent) if removed.contains(parent))
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
        self.databases.retain(|name, _| !removed.contains(name));
        self.uncertain_relations
            .retain(|name| !removed.contains(name));
    }
}
