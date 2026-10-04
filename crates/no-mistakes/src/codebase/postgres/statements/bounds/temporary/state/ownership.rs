//! Partition ownership transitions within one SQL source.
use super::{key, Dependency, State};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeSet;

impl State {
    pub fn restrict_blocks_drop(&self, names: &[String], catalog: Option<&SchemaCatalog>) -> bool {
        let mut removed: BTreeSet<_> = names
            .iter()
            .filter(|name| self.resolves_temporary(name, catalog))
            .map(|name| key(name))
            .collect();
        // A partition is owned by its parent even without CASCADE. A view on any
        // owned child blocks the whole DROP statement under PostgreSQL RESTRICT.
        loop {
            let before = removed.len();
            for (child, parent) in &self.partitions {
                if removed.contains(parent) {
                    removed.insert(child.clone());
                }
            }
            if removed.len() == before {
                break;
            }
        }
        self.relations.iter().any(|(name, dependencies)| {
            !removed.contains(name)
                && dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Temporary(source) if removed.contains(source))
                })
        })
    }

    pub fn partitioned_parent(&self, parent: &str) -> bool {
        self.matches_identity(parent) && self.partitioned.contains(&key(parent))
    }

    pub fn attach_partition(&mut self, parent: &str, child: &str, catalog: Option<&SchemaCatalog>) {
        if self.partitioned.contains(&key(parent))
            && self.resolves_temporary(parent, catalog)
            && self.resolves_temporary(child, catalog)
        {
            // PostgreSQL rejects attaching a relation that already belongs to a parent.
            self.partitions.entry(key(child)).or_insert(key(parent));
        }
    }

    pub fn attach_created_partition(&mut self, parent: &str, child: &str) {
        // The caller proved the parent partitioned before recording the new child.
        self.partitions.insert(key(child), key(parent));
    }

    pub fn detach_partition(&mut self, parent: &str, child: &str, catalog: Option<&SchemaCatalog>) {
        if self.resolves_temporary(parent, catalog) && self.resolves_temporary(child, catalog) {
            let child = key(child);
            if self
                .partitions
                .get(&child)
                .is_some_and(|owner| *owner == key(parent))
            {
                self.partitions.remove(&child);
            }
        }
    }
}
