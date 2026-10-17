use super::super::temporaries::TransientRoot;
use super::super::{Capture, Environment, Value};
use super::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Clone, Copy)]
pub(super) enum ModulePolicy {
    Referenced,
    AllModules,
}

pub(super) struct RootSeeds<'a> {
    pub modules: &'a FxHashMap<std::path::PathBuf, Environment>,
    pub module_policy: ModulePolicy,
    pub environments: &'a [Environment],
    pub arguments: &'a [u64],
    pub transient: &'a [TransientRoot],
}

impl<'a> RootSeeds<'a> {
    pub fn new(modules: &'a FxHashMap<std::path::PathBuf, Environment>) -> Self {
        Self {
            modules,
            module_policy: ModulePolicy::Referenced,
            environments: &[],
            arguments: &[],
            transient: &[],
        }
    }

    pub fn all_modules(modules: &'a FxHashMap<std::path::PathBuf, Environment>) -> Self {
        Self {
            module_policy: ModulePolicy::AllModules,
            ..Self::new(modules)
        }
    }
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
    fn value(
        &mut self,
        value: &Value,
        modules: &FxHashMap<std::path::PathBuf, Environment>,
        policy: ModulePolicy,
    ) {
        match value {
            // This policy seeds every module frame below. Its functions need
            // no repeated owner-path lookup; selected roots still follow each
            // actual owner, including functions inside nested containers.
            Value::Function(_, _, Capture::Module) if matches!(policy, ModulePolicy::AllModules) => {}
            Value::Function(_, path, capture) => self.pending_env.push(capture.environment(path, modules)),
            Value::Arguments(id) | Value::Array(id) => self.pending_args.push(*id),
            Value::Prefix(_, _, Some(id)) => {
                self.identities.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.value(value, modules, policy),
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
                    self.value(value, modules, policy);
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
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.reachability_walks", 1);
    }
    let mut binding_visits = 0;
    let mut found = Reachable::default();
    if matches!(roots.module_policy, ModulePolicy::AllModules) {
        found
            .pending_env
            .reserve(roots.environments.len() + roots.modules.len());
        found.pending_env.extend(roots.modules.values().copied());
    }
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
        found.value(value, roots.modules, roots.module_policy);
    }
    while !found.pending_env.is_empty() || !found.pending_args.is_empty() {
        while let Some(env) = found.pending_env.pop() {
            if found.environments.insert(env) {
                if let Some(origins) = captured.get(&env) {
                    found.pending_env.extend(origins.values().copied());
                }
                // All module owners are already seeded. Captured origins above
                // and mapped argument objects below remain dynamic side edges.
                if !(matches!(roots.module_policy, ModulePolicy::AllModules)
                    && scopes[env].is_module_only())
                {
                    if observer.is_some() {
                        binding_visits += scopes[env].len() as u64;
                    }
                    for value in scopes[env].values() {
                        found.value(value, roots.modules, roots.module_policy);
                    }
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
                    found.value(value, roots.modules, roots.module_policy);
                }
            }
        }
    }
    if let Some(observer) = observer {
        observer.increment(
            "postgres.annotation.reachability_binding_visits",
            binding_visits,
        );
    }
    found
}

#[cfg(test)]
#[path = "reachable/frontier_tests.rs"]
mod frontier_tests;

#[cfg(test)]
#[path = "reachable/frontier_work_tests.rs"]
mod frontier_work_tests;
