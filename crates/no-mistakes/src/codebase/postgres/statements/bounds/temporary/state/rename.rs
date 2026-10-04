//! Rename relation and schema dependencies without changing request ownership.
use super::{key, names::names_match, Dependency, State};
use crate::codebase::postgres::decoded_parts;
use std::collections::BTreeSet;

impl State {
    pub fn rename_schema(&mut self, old: &str, new: &str) {
        let old = decoded_parts(old);
        let new = decoded_parts(new);
        let ([old], [new]) = (old.as_slice(), new.as_slice()) else {
            return;
        };
        for dependencies in self
            .relations
            .values_mut()
            .chain(self.physical_views.values_mut())
        {
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
        let mut moved = std::collections::BTreeMap::new();
        for (mut name, dependencies) in std::mem::take(&mut self.physical_views) {
            if name.len() >= 2 && name[name.len() - 2] == *old {
                let schema = name.len() - 2;
                name[schema] = new.to_string();
            }
            moved
                .entry(name)
                .or_insert_with(BTreeSet::new)
                .extend(dependencies);
        }
        self.physical_views = moved;
    }

    pub fn rename(&mut self, old: &str, new: &str) {
        if !self.matches_identity(old) && self.possible_temporary(old).is_some() {
            // A physical namesake may have been renamed instead. Never carry a possibly
            // stale temporary identity into a later, more definite search path.
            self.remove(BTreeSet::from([key(old)]), true);
        }
        if !self.matches_identity(old) {
            let old = decoded_parts(old);
            let new = key(new);
            self.rename_physical(&old, &new);
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
        if let Some(database) = self.databases.remove(&old) {
            self.databases.insert(new.clone(), database);
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
            *dependencies = dependencies
                .iter()
                .map(|dependency| match dependency {
                    Dependency::Temporary(name) if name == &old => {
                        Dependency::Temporary(new.clone())
                    }
                    Dependency::ConditionalTemporary(name, database) if name == &old => {
                        Dependency::ConditionalTemporary(new.clone(), database.clone())
                    }
                    other => other.clone(),
                })
                .collect();
        }
    }
}
