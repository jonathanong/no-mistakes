mod dependencies;
mod imports;
mod memo;
mod state;
use super::{calls::scopes, Environment, Evaluator, Value};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

fn callback_depth(depth: u8) -> impl Iterator<Item = u8> {
    depth.checked_sub(1).into_iter()
}

#[derive(Default)]
struct CallbackState {
    objects: FxHashMap<u64, (u8, Vec<Value>)>,
    functions: FxHashMap<CallbackIdentity, (u8, Value, std::sync::Arc<state::Snapshot>)>,
    memo: memo::Memo,
}

pub(in crate::codebase::postgres::query_annotation) type CallbackIdentity =
    (PathBuf, u32, Environment, Vec<String>);

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        let outermost = self.active_callback_functions.is_none();
        self.active_callback_functions
            .get_or_insert_with(FxHashSet::default);
        self.active_callback_executions
            .get_or_insert_with(FxHashSet::default);
        let mut state = CallbackState::default();
        self.callback_values(arguments, depth, &mut state);
        // Only objects reached by this consumer participate. Each revisit
        // spends the existing depth budget, including cyclic installers.
        for revisit_depth in (0..depth).rev() {
            let mut changed = state
                .objects
                .iter()
                .filter_map(|(id, (budget, previous))| {
                    let live = self.live_callbacks(*id);
                    (live != *previous).then_some((*id, (*budget).min(revisit_depth), live))
                })
                .collect::<Vec<_>>();
            let functions = state
                .functions
                .iter()
                .map(|(key, (budget, value, before))| {
                    (key.clone(), *budget, value.clone(), before.clone())
                })
                .collect::<Vec<_>>();
            let mut comparison = memo::Memo::default();
            let mut callbacks = Vec::new();
            for (key, budget, value, before) in functions {
                if let Value::Function(function, path, captured) = &value {
                    let current = self.memoized_callback_snapshot(
                        path,
                        *captured,
                        function,
                        budget,
                        &mut comparison,
                    );
                    if current != before {
                        callbacks.push((key, budget.min(revisit_depth), value));
                    }
                }
            }
            if changed.is_empty() && callbacks.is_empty() {
                break;
            }
            callbacks.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            for (key, budget, value) in callbacks {
                self.active_callback_functions
                    .as_mut()
                    .unwrap()
                    .remove(&key);
                self.callback_values(std::slice::from_ref(&value), budget, &mut state);
            }
            changed.sort_unstable_by_key(|(id, _, _)| *id);
            for (id, budget, live) in changed {
                state.objects.get_mut(&id).unwrap().1 = live.clone();
                self.callback_values(&live, budget, &mut state);
            }
        }
        if outermost {
            self.active_callback_functions = None;
            self.active_callback_executions = None;
        }
    }

    fn live_callbacks(&self, id: u64) -> Vec<Value> {
        self.argument_slots(id)
            .filter(|(index, _)| !self.definite_deleted_argument_slots.contains(&(id, *index)))
            .map(|(_, value)| value.clone())
            .collect()
    }

    fn callback_values(&mut self, values: &[Value], depth: u8, visited: &mut CallbackState) {
        for value in values {
            match value {
                Value::Aggregate(values) | Value::Possible(values) => {
                    self.callback_values(values, depth, visited)
                }
                Value::Promise(value) | Value::Evaluated(value, _) => {
                    self.callback_values(std::slice::from_ref(value.as_ref()), depth, visited);
                }
                Value::Arguments(id)
                    if visited
                        .objects
                        .get(id)
                        .is_none_or(|(budget, _)| *budget < depth) =>
                {
                    let live = self.live_callbacks(*id);
                    visited.objects.insert(*id, (depth, live.clone()));
                    self.callback_values(&live, depth, visited);
                }
                Value::Function(function, path, captured) => {
                    let taints = self.invalidated_builders.len();
                    self.invalidate_captured(*captured, function);
                    if self.invalidated_builders.len() != taints {
                        visited.memo.clear();
                    }
                    for next_depth in callback_depth(depth) {
                        let key = (
                            path.clone(),
                            function.start,
                            *captured,
                            function.params.clone(),
                        );
                        if self
                            .active_callback_executions
                            .as_ref()
                            .unwrap()
                            .contains(&key)
                        {
                            continue;
                        }
                        if self
                            .active_callback_functions
                            .as_mut()
                            .expect("active opaque consumer")
                            .insert(key.clone())
                        {
                            self.active_callback_executions
                                .as_mut()
                                .unwrap()
                                .insert(key.clone());
                            let snapshot = self.memoized_callback_snapshot(
                                path,
                                *captured,
                                function,
                                depth,
                                &mut visited.memo,
                            );
                            visited
                                .functions
                                .insert(key.clone(), (depth, value.clone(), snapshot));
                            let mut locals = scopes::locals(&self.scopes[*captured], function);
                            if let Some(name) = &function.self_name {
                                locals.insert(name.clone(), Value::Unknown);
                            }
                            scopes::arguments(&mut locals, function, Value::Unknown);
                            for name in &function.params {
                                locals.insert(name.clone(), Value::Unknown);
                            }
                            // Opaque consumers do not supply a reference receiver.
                            scopes::bind_this(&mut locals, function, None);
                            let mut scope = self.environment(locals);
                            self.register_mappings(scope, *captured, function, None);
                            self.register_captured_bindings(scope, *captured, function);
                            let mutating = self.callback_may_mutate(function, path, scope);
                            if mutating {
                                visited.memo.clear();
                            }
                            let returned =
                                self.steps(&function.body, path, &mut scope, next_depth, false);
                            if mutating {
                                visited.memo.clear();
                            }
                            self.callback_values(
                                std::slice::from_ref(&returned),
                                next_depth,
                                visited,
                            );
                            self.active_callback_executions
                                .as_mut()
                                .unwrap()
                                .remove(&key);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "callbacks/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "callbacks/dependencies_tests.rs"]
mod dependencies_tests;
#[cfg(test)]
#[path = "callbacks/profile_tests.rs"]
mod profile_tests;

#[cfg(test)]
#[path = "callbacks/imported_tests.rs"]
mod imported_tests;

#[cfg(test)]
#[path = "callbacks/memo_tests.rs"]
mod memo_tests;
