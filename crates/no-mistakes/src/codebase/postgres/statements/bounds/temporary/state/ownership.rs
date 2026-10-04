//! Partition ownership transitions within one SQL source.
use super::{key, State};

impl State {
    pub fn partitioned_parent(&self, parent: &str) -> bool {
        self.matches_identity(parent) && self.partitioned.contains(&key(parent))
    }

    pub fn attach_partition(&mut self, parent: &str, child: &str) {
        if self.partitioned_parent(parent) && self.matches_identity(child) {
            // PostgreSQL rejects attaching a relation that already belongs to a parent.
            self.partitions.entry(key(child)).or_insert(key(parent));
        }
    }

    pub fn attach_created_partition(&mut self, parent: &str, child: &str) {
        // The caller proved the parent partitioned before recording the new child.
        self.partitions.insert(key(child), key(parent));
    }

    pub fn detach_partition(&mut self, parent: &str, child: &str) {
        if self.matches_identity(parent) && self.matches_identity(child) {
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
