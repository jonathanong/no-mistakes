use super::super::{Evaluator, Scope, Value};
use super::arena::Arena;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

type Updates = FxHashMap<u64, Value>;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn alternative_builder_baselines(&self, path: &Path) -> Updates {
        baseline(
            &self.scopes,
            Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.builder_updates,
            &self.definite_deleted_argument_slots,
            self.files[path].facts.precise_records,
        )
    }

    pub(in super::super) fn join_builder_updates(
        &self,
        joined: &mut Option<Updates>,
        baseline: &Updates,
        original_ids: u64,
        path: &Path,
    ) {
        join(
            joined,
            baseline,
            &self.builder_updates,
            original_ids,
            self.files[path].facts.precise_records,
        );
    }
    pub(in super::super) fn restore_builder_updates(
        &mut self,
        joined: Option<Updates>,
        previous: Updates,
        original_ids: u64,
    ) {
        self.builder_updates.retain(|id, _| *id >= original_ids);
        self.builder_updates.extend(joined.unwrap_or(previous));
    }
}

fn baseline(
    scopes: &[Scope],
    arena: Arena<'_>,
    updates: &Updates,
    definite: &FxHashSet<(u64, usize)>,
    precise: bool,
) -> Updates {
    let mut values = Updates::default();
    if precise {
        let mut visited = FxHashSet::default();
        for scope in scopes {
            for value in scope.values() {
                collect(value, arena, &mut visited, &mut values, definite);
            }
        }
        // Existing canonical updates include detached aliases and prior joins.
        values.extend(updates.clone());
    }
    values
}

fn collect(
    value: &Value,
    arena: Arena<'_>,
    visited: &mut FxHashSet<u64>,
    values: &mut Updates,
    definite: &FxHashSet<(u64, usize)>,
) {
    match value {
        Value::Prefix(_, _, Some(id)) => {
            let next = match values.get(id) {
                Some(previous) => {
                    super::super::concat::joined(vec![previous.clone(), value.clone()])
                }
                None => value.clone(),
            };
            values.insert(*id, next);
        }
        Value::Promise(value) | Value::Evaluated(value, _) => {
            collect(value, arena, visited, values, definite)
        }
        Value::Aggregate(children)
        | Value::References(children)
        | Value::Object(children)
        | Value::Record(children, _)
        | Value::Joined(children)
        | Value::Possible(children) => {
            for child in children {
                collect(child, arena, visited, values, definite);
            }
        }
        Value::Arguments(id) | Value::Array(id) if visited.insert(*id) => {
            for (index, child) in arena.objects[id].iter().enumerate().chain(
                arena
                    .extras
                    .get(id)
                    .into_iter()
                    .flat_map(|slots| slots.iter().map(|(index, value)| (*index, value))),
            ) {
                if !definite.contains(&(*id, index)) {
                    collect(child, arena, visited, values, definite);
                }
            }
        }
        _ => {}
    }
}

fn join(
    joined: &mut Option<Updates>,
    baseline: &Updates,
    updates: &Updates,
    original_ids: u64,
    precise: bool,
) {
    if !precise {
        return;
    }
    // Each arm starts from the original identity state, including unchanged arms.
    let mut current = baseline.clone();
    current.extend(
        updates
            .iter()
            .filter(|(id, _)| **id < original_ids)
            .map(|(id, value)| (*id, value.clone())),
    );
    if let Some(joined) = joined {
        for (id, previous) in joined.iter_mut() {
            let next = current.get(id).unwrap_or(&Value::Unknown);
            *previous = super::super::concat::joined(vec![previous.clone(), next.clone()]);
        }
        for (id, value) in current {
            joined
                .entry(id)
                .or_insert_with(|| super::super::concat::joined(vec![Value::Unknown, value]));
        }
    } else {
        *joined = Some(current);
    }
}
