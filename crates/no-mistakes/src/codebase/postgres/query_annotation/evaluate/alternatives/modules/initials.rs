mod remap;
use super::super::super::{Environment, Evaluator, Value};
use super::super::{arena::Arena, reachable};
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation) struct Initials {
    pub base: usize,
    pub known: FxHashSet<PathBuf>,
    frames: FxHashMap<Environment, FxHashMap<String, Value>>,
    objects: FxHashMap<u64, Vec<Value>>,
    extras: FxHashMap<u64, BTreeMap<usize, Value>>,
    mapped: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: FxHashMap<Environment, FxHashMap<String, Environment>>,
    fresh: FxHashMap<Environment, FxHashSet<String>>,
    identities: FxHashSet<u64>,
    updates: FxHashMap<u64, Value>,
    invalidated: FxHashSet<u64>,
    deleted: FxHashSet<(u64, Option<usize>)>,
    definite: FxHashSet<(u64, usize)>,
    disconnected: FxHashSet<(u64, usize)>,
}
impl Initials {
    pub fn arguments(&self) -> impl Iterator<Item = u64> + '_ {
        self.objects.keys().copied()
    }
    pub fn roots(&self) -> impl Iterator<Item = Environment> + '_ {
        self.frames.keys().copied()
    }
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn record_alternative_module_initial(
        &mut self,
        path: &Path,
        env: Environment,
    ) {
        if self.active_module_initials.is_empty() {
            return;
        }
        let live = reachable::collect_from_roots(
            &self.scopes,
            &[env],
            &[],
            Arena {
                objects: &self.argument_objects,
                extras: &self.argument_extra_slots,
            },
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        for snapshot in &mut self.active_module_initials {
            if snapshot.known.contains(path) {
                continue;
            }
            snapshot.known.insert(path.to_path_buf());
            for frame in live
                .environments
                .iter()
                .filter(|frame| **frame >= snapshot.base)
            {
                snapshot
                    .frames
                    .entry(*frame)
                    .or_insert_with(|| self.scopes[*frame].clone());
                if let Some(value) = self.mapped_arguments.get(frame) {
                    snapshot
                        .mapped
                        .entry(*frame)
                        .or_insert_with(|| value.clone());
                }
                if let Some(value) = self.captured_bindings.get(frame) {
                    snapshot
                        .captured
                        .entry(*frame)
                        .or_insert_with(|| value.clone());
                }
                if let Some(value) = self.fresh_mapped_parameters.get(frame) {
                    snapshot
                        .fresh
                        .entry(*frame)
                        .or_insert_with(|| value.clone());
                }
            }
            for id in &live.arguments {
                snapshot
                    .objects
                    .entry(*id)
                    .or_insert_with(|| self.argument_objects[id].clone());
                if let Some(value) = self.argument_extra_slots.get(id) {
                    snapshot.extras.entry(*id).or_insert_with(|| value.clone());
                }
            }
            snapshot.identities.extend(live.identities.iter().copied());
            snapshot.updates.extend(
                self.builder_updates
                    .iter()
                    .filter(|(id, _)| live.identities.contains(id))
                    .map(|(id, value)| (*id, value.clone())),
            );
            snapshot.invalidated.extend(
                self.invalidated_builders
                    .intersection(&live.identities)
                    .copied(),
            );
            snapshot.deleted.extend(
                self.deleted_argument_slots
                    .iter()
                    .filter(|(id, _)| live.arguments.contains(id))
                    .copied(),
            );
            snapshot.definite.extend(
                self.definite_deleted_argument_slots
                    .iter()
                    .filter(|(id, _)| live.arguments.contains(id))
                    .copied(),
            );
            snapshot.disconnected.extend(
                self.disconnected_argument_slots
                    .iter()
                    .filter(|(id, _)| live.arguments.contains(id))
                    .copied(),
            );
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn restore_alternative_modules(
        &mut self,
    ) {
        let snapshot = self
            .active_module_initials
            .last()
            .expect("active alternatives");
        for (env, values) in &snapshot.frames {
            self.scopes[*env].clone_from(values);
        }
        for (id, values) in &snapshot.objects {
            self.argument_objects.insert(*id, values.clone());
            self.argument_extra_slots.remove(id);
        }
        self.argument_extra_slots.extend(snapshot.extras.clone());
        self.mapped_arguments.extend(snapshot.mapped.clone());
        self.captured_bindings.extend(snapshot.captured.clone());
        for env in snapshot.frames.keys() {
            self.fresh_mapped_parameters.remove(env);
        }
        self.fresh_mapped_parameters.extend(snapshot.fresh.clone());
        self.builder_updates
            .retain(|id, _| !snapshot.identities.contains(id));
        self.builder_updates.extend(snapshot.updates.clone());
        self.invalidated_builders
            .extend(snapshot.invalidated.iter().copied());
        self.deleted_argument_slots
            .extend(snapshot.deleted.iter().copied());
        self.definite_deleted_argument_slots
            .extend(snapshot.definite.iter().copied());
        self.disconnected_argument_slots
            .extend(snapshot.disconnected.iter().copied());
    }
}

#[cfg(test)]
mod tests;
