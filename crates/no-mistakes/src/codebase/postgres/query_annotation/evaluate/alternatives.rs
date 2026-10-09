//! Alternative arms share input facts, never each other's mutable effects.
mod frames;
mod merge;
mod reachable;
mod values;
use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashSet;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(super) fn alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let scopes = self.scopes.clone();
        let modules = self.modules.clone();
        let original = self.invalidated_builders.clone();
        let objects = self.argument_objects.clone();
        let definite = self.definite_deleted_argument_slots.clone();
        let mut definite_common = None;
        let mut private_definite = FxHashSet::default();
        let mut joined = crate::fx::FxHashMap::default();
        let deleted = self.deleted_argument_slots.clone();
        let mut deleted_changed = deleted.clone();
        let mut returned = Vec::new();
        let mut changed = FxHashSet::default();
        for arm in arms {
            // Keep arm-created frames alive for callbacks returned from helpers.
            self.scopes[..scopes.len()].clone_from_slice(&scopes);
            self.modules.clone_from(&modules);
            self.invalidated_builders.clone_from(&original);
            self.deleted_argument_slots.clone_from(&deleted);
            self.definite_deleted_argument_slots.clone_from(&definite);
            for (id, slots) in &objects {
                self.argument_objects.insert(*id, slots.clone());
            }
            returned.push(self.expr(arm, path, env, depth, generic));
            deleted_changed.extend(self.deleted_argument_slots.iter().copied());
            changed.extend(self.invalidated_builders.iter().copied());
            merge::definite(
                &mut definite_common,
                &mut private_definite,
                &self.definite_deleted_argument_slots,
                &objects,
            );
            for (before, after) in scopes.iter().zip(&self.scopes) {
                for (name, value) in before {
                    // Evaluation replaces bindings but never removes original keys.
                    values::changes(
                        value,
                        &after[name],
                        &objects,
                        &self.argument_objects,
                        &mut changed,
                        &self.definite_deleted_argument_slots,
                    );
                }
            }
            merge::objects(&mut joined, &self.argument_objects, &objects);
            self.scopes[..scopes.len()].clone_from_slice(&scopes);
            for (id, slots) in &joined {
                self.argument_objects.insert(*id, slots.clone());
            }
            frames::compact(
                &mut self.scopes,
                scopes.len(),
                &mut returned,
                &mut self.mapped_arguments,
                &mut self.argument_objects,
            );
            joined = objects
                .keys()
                .filter_map(|id| {
                    self.argument_objects
                        .get(id)
                        .map(|slots| (*id, slots.clone()))
                })
                .collect();
        }
        self.scopes[..scopes.len()].clone_from_slice(&scopes);
        self.modules = modules;
        self.invalidated_builders = original;
        self.deleted_argument_slots = deleted_changed;
        let mut final_definite = definite_common.unwrap_or(definite.clone());
        final_definite.extend(private_definite);
        let values = changed
            .into_iter()
            .map(|id| Value::Prefix(String::new(), false, Some(id)))
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        // Restored bindings must retain any prior opaque parameter update before
        // the common deletion disconnects them from future argument mutations.
        self.definite_deleted_argument_slots = definite.clone();
        for (id, index) in final_definite.difference(&definite) {
            self.disconnect_mapped_slot(*id, *index);
        }
        self.definite_deleted_argument_slots = final_definite;
        // Speculative identities with no surviving aliases cannot affect later reads.
        frames::prune_state(
            &self.scopes,
            &returned,
            &mut self.deleted_argument_slots,
            &mut self.definite_deleted_argument_slots,
            &mut self.invalidated_builders,
            &self.mapped_arguments,
            &mut self.argument_objects,
        );
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        Value::Aggregate(returned)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod deletion_tests;
