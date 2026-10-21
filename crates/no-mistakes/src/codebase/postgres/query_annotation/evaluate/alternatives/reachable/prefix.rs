use super::super::super::{references::Edges, Scope};
use crate::fx::FxHashMap;
use std::hash::Hash;

#[derive(Default)]
struct Counts<T> {
    values: FxHashMap<T, usize>,
}

impl<T: Eq + Hash + Copy> Counts<T> {
    fn add(&mut self, values: impl Iterator<Item = T>) {
        for value in values {
            *self.values.entry(value).or_default() += 1;
        }
    }

    fn remove(&mut self, values: impl Iterator<Item = T>) {
        for value in values {
            let count = self.values.get_mut(&value).expect("cached direct root");
            *count -= 1;
            if *count == 0 {
                self.values.remove(&value);
            }
        }
    }
}

/// Owned Scope snapshots prevent pointer reuse. Counts remove the last root,
/// rather than accumulating stale liveness across alternative arms.
#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) struct Prefix {
    scopes: Vec<Scope>,
    environments: Counts<usize>,
    arguments: Counts<u64>,
    identities: Counts<u64>,
}

impl Prefix {
    pub(super) fn refresh(&mut self, scopes: &[Scope]) {
        if scopes.len() != self.scopes.len() {
            *self = Self::default();
        }
        for (index, scope) in scopes.iter().enumerate() {
            if self
                .scopes
                .get(index)
                .is_some_and(|old| old.same_bindings(scope))
            {
                continue;
            }
            if let Some(old) = self.scopes.get(index) {
                let edges = old.references();
                self.environments.remove(edges.environments.iter().copied());
                self.arguments.remove(edges.arguments.iter().copied());
                self.identities.remove(edges.identities.iter().copied());
            }
            let Edges {
                environments,
                arguments,
                identities,
            } = scope.references();
            self.environments.add(environments.iter().copied());
            self.arguments.add(arguments.iter().copied());
            self.identities.add(identities.iter().copied());
            if index == self.scopes.len() {
                self.scopes.push(scope.clone());
            } else {
                self.scopes[index] = scope.clone();
            }
        }
    }

    pub(super) fn seed(&self, found: &mut super::Reachable) {
        found.environments.extend(0..self.scopes.len());
        found
            .pending_env
            .extend(self.environments.values.keys().copied());
        found
            .pending_args
            .extend(self.arguments.values.keys().copied());
        found
            .identities
            .extend(self.identities.values.keys().copied());
    }
}
