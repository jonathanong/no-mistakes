use super::super::{Environment, Evaluator, Function, Value};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default, PartialEq, Eq)]
pub(in crate::codebase::postgres::query_annotation::evaluate) struct Snapshot {
    scopes: FxHashMap<Environment, FxHashMap<String, Value>>,
    objects: FxHashMap<u64, Vec<Value>>,
    extras: FxHashMap<u64, BTreeMap<usize, Value>>,
    updates: FxHashMap<u64, Value>,
    possible: FxHashSet<(u64, Option<usize>)>,
    definite: FxHashSet<(u64, usize)>,
    disconnected: FxHashSet<(u64, usize)>,
    recreated: FxHashSet<(u64, usize)>,
    invalidated: FxHashSet<u64>,
    fresh: FxHashMap<Environment, FxHashSet<String>>,
}

pub(super) struct View<'a> {
    scopes: &'a [Scope],
    objects: &'a FxHashMap<u64, Vec<Value>>,
    extras: &'a FxHashMap<u64, BTreeMap<usize, Value>>,
    updates: &'a FxHashMap<u64, Value>,
    possible: &'a FxHashSet<(u64, Option<usize>)>,
    definite: &'a FxHashSet<(u64, usize)>,
    disconnected: &'a FxHashSet<(u64, usize)>,
    recreated: &'a FxHashSet<(u64, usize)>,
    invalidated: &'a FxHashSet<u64>,
    fresh: &'a FxHashMap<Environment, FxHashSet<String>>,
    mapped: &'a FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &'a FxHashMap<Environment, FxHashMap<String, Environment>>,
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn callback_view(&self) -> View<'_> {
        View {
            scopes: &self.scopes,
            objects: &self.argument_objects,
            extras: &self.argument_extra_slots,
            updates: &self.builder_updates,
            possible: &self.deleted_argument_slots,
            definite: &self.definite_deleted_argument_slots,
            disconnected: &self.disconnected_argument_slots,
            recreated: &self.recreated_argument_slots,
            invalidated: &self.invalidated_builders,
            fresh: &self.fresh_mapped_parameters,
            mapped: &self.mapped_arguments,
            captured: &self.captured_bindings,
        }
    }
}
impl View<'_> {
    pub(super) fn snapshot(
        &self,
        path: &Path,
        captured: Environment,
        function: &Function,
        imports: &FxHashMap<PathBuf, FxHashMap<String, Value>>,
    ) -> (Snapshot, FxHashSet<(PathBuf, String)>) {
        let mut missing = FxHashSet::default();
        let mut found = Snapshot::default();
        let mut functions = vec![(path, captured, function)];
        let mut visited = FxHashSet::default();
        let mut values = Vec::new();
        while !functions.is_empty() || !values.is_empty() {
            while let Some((path, env, function)) = functions.pop() {
                if !visited.insert((path, env, function.start)) {
                    continue;
                }
                let reads = super::dependencies::names(function);
                for (name, read_value) in reads
                    .values
                    .into_iter()
                    .map(|name| (name, true))
                    .chain(reads.identities.into_iter().map(|name| (name, false)))
                {
                    let origin = self
                        .captured
                        .get(&env)
                        .and_then(|origins| origins.get(&name))
                        .copied()
                        .unwrap_or(env);
                    let value = self.scopes[origin]
                        .get(&name)
                        .or_else(|| imports.get(path).and_then(|bindings| bindings.get(&name)));
                    if value.is_none() {
                        missing.insert((path.to_path_buf(), name.clone()));
                    }
                    if let Some(value) = value {
                        found
                            .scopes
                            .entry(origin)
                            .or_default()
                            .insert(name.clone(), value.clone());
                        if !read_value {
                            continue;
                        }
                        if self
                            .fresh
                            .get(&origin)
                            .is_some_and(|names| names.contains(&name))
                        {
                            found.fresh.entry(origin).or_default().insert(name.clone());
                        }
                        for (id, params) in self.mapped.get(&origin).into_iter().flatten() {
                            if params.iter().any(|parameter| parameter == &name) {
                                self.snapshot_argument(*id, &mut found, &mut values);
                            }
                        }
                        values.push(value);
                    }
                }
            }
            while let Some(value) = values.pop() {
                match value {
                    Value::Function(function, path, env) => {
                        functions.push((path.as_path(), *env, function))
                    }
                    Value::Arguments(id) => self.snapshot_argument(*id, &mut found, &mut values),
                    Value::Prefix(_, _, Some(id)) => {
                        if let Some(value) = self.updates.get(id) {
                            found.updates.insert(*id, value.clone());
                        }
                        if self.invalidated.contains(id) {
                            found.invalidated.insert(*id);
                        }
                    }
                    Value::Aggregate(values_)
                    | Value::Joined(values_)
                    | Value::Possible(values_) => values.extend(values_),
                    Value::Promise(value) | Value::Evaluated(value, _) => values.push(value),
                    _ => {}
                }
            }
        }
        (found, missing)
    }
    fn snapshot_argument<'a>(&'a self, id: u64, found: &mut Snapshot, values: &mut Vec<&'a Value>) {
        if found.objects.contains_key(&id) {
            return;
        }
        let slots = &self.objects[&id];
        found.objects.insert(id, slots.clone());
        if let Some(extras) = self.extras.get(&id) {
            found.extras.insert(id, extras.clone());
        }
        if self.invalidated.contains(&id) {
            found.invalidated.insert(id);
        }
        if self.possible.contains(&(id, None)) {
            found.possible.insert((id, None));
        }
        for (index, value) in self.objects[&id].iter().enumerate().chain(
            self.extras
                .get(&id)
                .into_iter()
                .flat_map(|slots| slots.iter().map(|(index, value)| (*index, value))),
        ) {
            if self.possible.contains(&(id, Some(index))) {
                found.possible.insert((id, Some(index)));
            }
            if self.definite.contains(&(id, index)) {
                found.definite.insert((id, index));
            }
            if self.disconnected.contains(&(id, index)) {
                found.disconnected.insert((id, index));
            }
            if self.recreated.contains(&(id, index)) {
                found.recreated.insert((id, index));
            }
            values.push(value);
        }
    }
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
