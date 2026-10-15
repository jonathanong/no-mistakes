use super::super::temporaries::TransientRoot;
use super::super::{Environment, Value};
use super::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct RootSeeds<'a> {
    pub environments: &'a [Environment],
    pub arguments: &'a [u64],
    pub transient: &'a [TransientRoot],
}

#[derive(Default)]
pub(super) struct Reachable {
    pub environments: FxHashSet<Environment>,
    pub arguments: FxHashSet<u64>,
    pub identities: FxHashSet<u64>,
    pending_env: Vec<Environment>,
    pending_args: Vec<u64>,
    visited_values: FxHashSet<*const ()>,
}

impl Reachable {
    fn value(&mut self, value: &Value) {
        match value {
            Value::Function(_, _, env) => self.pending_env.push(*env),
            Value::Arguments(id) | Value::Array(id) => self.pending_args.push(*id),
            Value::Prefix(_, _, Some(id)) => {
                self.identities.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.value(value),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values)
                // Functions carry environments without being builder/argument
                // references. Both immutable summaries must be empty to skip.
                if (values.contains_reference() || values.contains_environment())
                    && self.visited_values.insert(values.identity()) =>
            {
                for value in values {
                    self.value(value);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "reachable/tests.rs"]
mod tests;

pub(super) fn collect_from_roots(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Reachable {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.reachability_walks", 1);
    }
    let mut found = Reachable::default();
    found.pending_env.extend(roots.environments.iter().copied());
    found.pending_args.extend(roots.arguments.iter().copied());
    for root in roots.transient {
        match root {
            TransientRoot::Argument(id) => found.pending_args.push(*id),
            TransientRoot::Builder(id) => {
                found.identities.insert(*id);
            }
            TransientRoot::Environment(env) => found.pending_env.push(*env),
        }
    }
    for value in returned {
        found.value(value);
    }
    while !found.pending_env.is_empty() || !found.pending_args.is_empty() {
        while let Some(env) = found.pending_env.pop() {
            if found.environments.insert(env) {
                if let Some(origins) = captured.get(&env) {
                    found.pending_env.extend(origins.values().copied());
                }
                for value in scopes[env].values() {
                    found.value(value);
                }
                if let Some(bindings) = mapped.get(&env) {
                    found
                        .pending_args
                        .extend(bindings.iter().map(|(id, _)| *id));
                }
            }
        }
        while let Some(id) = found.pending_args.pop() {
            if found.arguments.insert(id) {
                found.identities.insert(id);
                for value in arena.objects[&id].iter().chain(
                    arena
                        .extras
                        .get(&id)
                        .into_iter()
                        .flat_map(|slots| slots.values()),
                ) {
                    found.value(value);
                }
            }
        }
    }
    found
}
