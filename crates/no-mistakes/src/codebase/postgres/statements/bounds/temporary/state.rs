//! Request-local SQL relation identities and dependency closure.
mod ownership;
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::statements::{
    SqlBoundItemKind, SqlBoundQuery, SqlPinSource, SqlPossibleTemporary,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Dependency {
    Temporary(String),
    Physical(Vec<String>),
}

// An unqualified reference has no schema identity here: invalidate possible dependents
// conservatively. Fully qualified identities must still agree on their schema.
fn names_match(left: &[String], right: &[String]) -> bool {
    let count = left.len().min(right.len());
    // decoded_parts always returns at least one part, even for an empty spelling.
    left[left.len() - count..] == right[right.len() - count..]
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
    pub fn dependency(&self, name: &str) -> Dependency {
        if self.contains(name) {
            Dependency::Temporary(key(name))
        } else {
            Dependency::Physical(decoded_parts(name))
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
                    out.insert(self.dependency(name));
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
    pub fn drop_schema(&mut self, name: &str) {
        let parts = decoded_parts(name);
        let [schema] = parts.as_slice() else {
            return;
        };
        let removed = self
            .relations
            .iter()
            .filter(|(_, dependencies)| {
                dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Physical(source)
                    if source.len() >= 2 && source[source.len() - 2] == *schema)
                })
            })
            .map(|(name, _)| name.clone())
            .collect();
        self.remove(removed, true);
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
    pub fn rename_schema(&mut self, old: &str, new: &str) {
        let old = decoded_parts(old);
        let new = decoded_parts(new);
        let ([old], [new]) = (old.as_slice(), new.as_slice()) else {
            return;
        };
        for dependencies in self.relations.values_mut() {
            *dependencies = dependencies
                .iter()
                .map(|dependency| match dependency {
                    Dependency::Physical(parts)
                        if parts.len() >= 2 && parts[parts.len() - 2] == *old =>
                    {
                        let mut renamed = parts.clone();
                        let schema = renamed.len() - 2;
                        renamed[schema] = new.to_string();
                        Dependency::Physical(renamed)
                    }
                    other => other.clone(),
                })
                .collect();
        }
    }
    pub fn rename(&mut self, old: &str, new: &str) {
        if !self.contains(old) && self.possible_temporary(old).is_some() {
            // A physical namesake may have been renamed instead. Never carry a possibly
            // stale temporary identity into a later, more definite search path.
            self.remove(BTreeSet::from([key(old)]), true);
        }
        if !self.contains(old) {
            let old = decoded_parts(old);
            let new = key(new);
            for dependencies in self.relations.values_mut() {
                *dependencies = dependencies
                    .iter()
                    .flat_map(|dependency| match dependency {
                        Dependency::Physical(parts) if names_match(parts, &old) => {
                            let mut renamed = parts.clone();
                            renamed.pop();
                            renamed.push(new.clone());
                            let mut candidates = vec![Dependency::Physical(renamed)];
                            // Without an exact schema match the rename may refer to a namesake.
                            if parts.len() == 1 || parts != &old {
                                candidates.push(dependency.clone());
                            }
                            candidates
                        }
                        other => vec![other.clone()],
                    })
                    .collect();
            }
            return;
        }
        let old = key(old);
        let new = key(new);
        if self.on_commit_drop.remove(&old) {
            self.on_commit_drop.insert(new.clone());
        }
        let dependencies = self.relations.remove(&old).unwrap_or_default();
        self.relations.insert(new.clone(), dependencies);
        if let Some(parent) = self.partitions.remove(&old) {
            self.partitions.insert(new.clone(), parent);
        }
        if self.partitioned.remove(&old) {
            self.partitioned.insert(new.clone());
        }
        for parent in self.partitions.values_mut() {
            if *parent == old {
                *parent = new.clone();
            }
        }
        for dependencies in self.relations.values_mut() {
            if dependencies.remove(&Dependency::Temporary(old.clone())) {
                dependencies.insert(Dependency::Temporary(new.clone()));
            }
        }
    }
}
