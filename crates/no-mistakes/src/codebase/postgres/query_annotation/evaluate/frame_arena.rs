//! Stable frame handles. Reusing a vacant slot never reuses its prior identity.
use super::{Environment, Scope};
use std::ops::{Index, IndexMut};

mod release;
mod storage;
use storage::Storage;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) struct FrameCheckpoint {
    pub(super) birth: u64,
    pub(super) allocation_log_index: usize,
}

#[derive(Clone, Default)]
pub(in crate::codebase::postgres::query_annotation) struct ScopeArena {
    storage: Storage,
    allocated: Vec<Environment>,
    next_birth: u64,
}

impl ScopeArena {
    pub(super) fn allocate(&mut self, scope: Scope) -> Environment {
        let env = self.storage.insert(scope, self.next_birth);
        self.next_birth += 1;
        self.allocated.push(env);
        env
    }

    pub(super) fn checkpoint(&self) -> FrameCheckpoint {
        FrameCheckpoint {
            birth: self.next_birth,
            allocation_log_index: self.allocated.len(),
        }
    }

    pub(super) fn is_before(&self, env: Environment, checkpoint: FrameCheckpoint) -> bool {
        self.storage
            .birth(env)
            .is_some_and(|birth| birth < checkpoint.birth)
    }

    pub(super) fn birth(&self, env: Environment) -> u64 {
        self.storage.birth(env).expect("live promoted environment")
    }

    pub(super) fn allocated_since(
        &self,
        checkpoint: FrameCheckpoint,
    ) -> impl Iterator<Item = Environment> + '_ {
        self.allocated[checkpoint.allocation_log_index..]
            .iter()
            .copied()
            .filter(|env| self.get(*env).is_some())
    }

    pub(super) fn get(&self, env: Environment) -> Option<&Scope> {
        self.storage.get(env)
    }

    pub(super) fn get_mut(&mut self, env: Environment) -> Option<&mut Scope> {
        self.storage.get_mut(env)
    }

    pub(super) fn release(&mut self, env: Environment) -> Option<Scope> {
        self.storage.remove(env)
    }

    pub(super) fn ids(&self) -> impl Iterator<Item = Environment> + '_ {
        self.storage.ids()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.ids().map(|env| &self[env])
    }

    /// Called only after all checkpoint users have released their log positions.
    pub(super) fn clear_allocation_log(&mut self) {
        self.allocated.clear();
        if self.allocated.capacity() > self.storage.len().saturating_mul(4).max(16) {
            self.allocated.shrink_to_fit();
        }
    }
}

impl From<Vec<Scope>> for ScopeArena {
    fn from(scopes: Vec<Scope>) -> Self {
        let mut arena = Self::default();
        for scope in scopes {
            arena.allocate(scope);
        }
        arena
    }
}

impl Index<Environment> for ScopeArena {
    type Output = Scope;
    fn index(&self, env: Environment) -> &Self::Output {
        self.get(env).expect("live environment handle")
    }
}

impl IndexMut<Environment> for ScopeArena {
    fn index_mut(&mut self, env: Environment) -> &mut Self::Output {
        self.get_mut(env).expect("live environment handle")
    }
}

#[cfg(test)]
#[path = "frame_arena/tests/accessors.rs"]
mod test_accessors;
