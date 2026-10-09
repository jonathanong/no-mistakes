//! Alternative arms share input facts, never each other's mutable effects.
mod bindings;
pub(super) mod frames;
mod freshness;
mod merge;
mod prune;
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
        let captured = self.captured_bindings.clone();
        let mut joined_scopes: Option<Vec<crate::fx::FxHashMap<String, Value>>> = None;
        let modules = self.modules.clone();
        let updates = self.builder_updates.clone();
        let original_ids = self.next_builder;
        let original = self.invalidated_builders.clone();
        let objects = self.argument_objects.clone();
        let definite = self.definite_deleted_argument_slots.clone();
        let fresh = self.fresh_mapped_parameters.clone();
        let mut fresh_joined = fresh.clone();
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
            self.builder_updates.retain(|id, _| *id >= original_ids);
            self.builder_updates.extend(updates.clone());
            self.captured_bindings
                .retain(|frame, _| *frame >= scopes.len());
            self.captured_bindings.extend(captured.clone());
            self.rebuild_captured_readers();
            self.rebuild_mapped_argument_owners();
            self.invalidated_builders.clone_from(&original);
            self.deleted_argument_slots.clone_from(&deleted);
            self.definite_deleted_argument_slots.clone_from(&definite);
            freshness::restore(&mut self.fresh_mapped_parameters, scopes.len(), &fresh);
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
            freshness::unchanged(
                &mut fresh_joined,
                &self.fresh_mapped_parameters,
                &scopes,
                &self.scopes,
            );
            merge::objects(&mut joined, &self.argument_objects, &objects);
            if let Some(joined) = &mut joined_scopes {
                bindings::join(joined, &self.scopes, &scopes);
            } else {
                joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
            }
            self.scopes[..scopes.len()].clone_from_slice(joined_scopes.as_ref().unwrap());
            for (id, slots) in &joined {
                self.argument_objects.insert(*id, slots.clone());
            }
            frames::compact(
                &mut self.scopes,
                scopes.len(),
                &mut returned,
                &mut self.mapped_arguments,
                &mut self.argument_objects,
                &mut self.fresh_mapped_parameters,
                &mut self.captured_bindings,
            );
            self.rebuild_captured_readers();
            self.rebuild_mapped_argument_owners();
            joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
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
        self.builder_updates.retain(|id, _| *id >= original_ids);
        self.builder_updates.extend(updates);
        self.captured_bindings
            .retain(|frame, _| *frame >= scopes.len());
        self.captured_bindings.extend(captured);
        self.rebuild_captured_readers();
        self.rebuild_mapped_argument_owners();
        if let Some(joined) = joined_scopes {
            self.scopes[..scopes.len()].clone_from_slice(&joined);
        }
        freshness::restore(
            &mut self.fresh_mapped_parameters,
            scopes.len(),
            &fresh_joined,
        );
        self.invalidated_builders = original;
        self.deleted_argument_slots = deleted_changed;
        let mut final_definite = definite_common.unwrap_or(definite.clone());
        final_definite.extend(private_definite);
        // These identities already represent actual arm effects. Replaying an
        // arguments escape would wrongly taint values assigned after that escape.
        self.invalidated_builders.extend(changed.iter().copied());
        values::apply_taint(&mut self.scopes, &changed);
        // Restored bindings must retain any prior opaque parameter update before
        // the common deletion disconnects them from future argument mutations.
        self.definite_deleted_argument_slots = definite.clone();
        for (id, index) in final_definite.difference(&definite) {
            self.disconnect_mapped_slot(*id, *index);
        }
        self.definite_deleted_argument_slots = final_definite;
        self.refresh_captured_bindings();
        // Speculative identities with no surviving aliases cannot affect later reads.
        frames::prune_state(
            &self.scopes,
            &returned,
            frames::MutationState {
                deleted: &mut self.deleted_argument_slots,
                definite: &mut self.definite_deleted_argument_slots,
                invalidated: &mut self.invalidated_builders,
            },
            &self.mapped_arguments,
            &mut self.argument_objects,
            &self.captured_bindings,
        );
        let live = reachable::collect(
            &self.scopes,
            self.scopes.len(),
            &returned,
            &self.argument_objects,
            &self.mapped_arguments,
            &self.captured_bindings,
        );
        self.builder_updates
            .retain(|id, _| live.identities.contains(id));
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        Value::Aggregate(returned)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod deletion_tests;
