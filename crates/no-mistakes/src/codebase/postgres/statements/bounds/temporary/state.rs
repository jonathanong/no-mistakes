//! Request-local SQL relation identities and dependency closure.
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery, SqlPinSource};
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
    pub fn dependencies(&self, query: &SqlBoundQuery, out: &mut BTreeSet<Dependency>) {
        for item in &query.items {
            match &item.kind {
                SqlBoundItemKind::Table(name) => {
                    out.insert(if self.contains(name) {
                        Dependency::Temporary(key(name))
                    } else {
                        Dependency::Physical(decoded_parts(name))
                    });
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
        if self.contains(name) {
            self.remove(BTreeSet::from([key(name)]), cascade);
        } else if cascade {
            let parts = decoded_parts(name);
            let removed = self.relations.iter().filter(|(_, dependencies)| {
                dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Physical(source) if names_match(source, &parts))
                })
            }).map(|(name, _)| name.clone()).collect();
            self.remove(removed, true);
        }
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
        if cascade {
            loop {
                let before = removed.len();
                for (name, dependencies) in &self.relations {
                    if dependencies
                        .iter()
                        .any(|dependency| matches!(dependency, Dependency::Temporary(name) if removed.contains(name)))
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
        for dependencies in self.relations.values_mut() {
            if dependencies.remove(&Dependency::Temporary(old.clone())) {
                dependencies.insert(Dependency::Temporary(new.clone()));
            }
        }
    }
}
