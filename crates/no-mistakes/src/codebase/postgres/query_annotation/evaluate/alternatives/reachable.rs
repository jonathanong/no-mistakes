use super::super::{Environment, Value};
use super::arena::Arena;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct Reachable {
    pub environments: FxHashSet<Environment>,
    pub arguments: FxHashSet<u64>,
    pub identities: FxHashSet<u64>,
    pending_env: Vec<Environment>,
    pending_args: Vec<u64>,
}

impl Reachable {
    fn value(&mut self, value: &Value) {
        match value {
            Value::Function(_, _, env) => self.pending_env.push(*env),
            Value::Arguments(id) => self.pending_args.push(*id),
            Value::Prefix(_, _, Some(id)) => {
                self.identities.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.value(value),
            Value::Aggregate(values) | Value::Possible(values) => {
                for value in values {
                    self.value(value);
                }
            }
            _ => {}
        }
    }
}

/// Scope and argument identities are separate visited sets, so mutually captured
/// callbacks and self-referential arguments remain finite.
pub(super) fn collect(
    scopes: &[FxHashMap<String, Value>],
    originals: usize,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Reachable {
    let roots = (0..originals).collect::<Vec<_>>();
    collect_from_roots(scopes, &roots, returned, arena, mapped, captured)
}

pub(super) fn collect_from_roots(
    scopes: &[FxHashMap<String, Value>],
    roots: &[Environment],
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Reachable {
    let mut found = Reachable::default();
    found.pending_env.extend(roots.iter().copied());
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
