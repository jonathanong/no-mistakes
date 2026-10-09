//! Alternative arms restore input state and join effects conservatively.
//! Imported modules keep one request-local initialization identity.
mod arena;
mod bindings;
mod extras;
pub(super) mod frames;
mod freshness;
mod merge;
mod modules;
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
        let mut modules = self.modules.clone();
        let mut module_states = modules::States::default();
        let extra_slots = self.argument_extra_slots.clone();
        let mut joined_extras = None;
        let updates = self.builder_updates.clone();
        let original_ids = self.next_builder;
        let original = self.invalidated_builders.clone();
        let objects = self.argument_objects.clone();
        let definite = self.definite_deleted_argument_slots.clone();
        let disconnected = self.disconnected_argument_slots.clone();
        let mut disconnected_common = None;
        let mut private_disconnected = FxHashSet::default();
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
            module_states.restore(&mut self.scopes, &modules);
            self.argument_extra_slots
                .retain(|id, _| !objects.contains_key(id));
            self.argument_extra_slots.extend(extra_slots.clone());
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
            self.disconnected_argument_slots.clone_from(&disconnected);
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
            merge::definite(
                &mut disconnected_common,
                &mut private_disconnected,
                &self.disconnected_argument_slots,
                &objects,
            );
            for (before, after) in scopes.iter().zip(&self.scopes) {
                for (name, value) in before {
                    // Evaluation replaces bindings but never removes original keys.
                    values::changes(
                        value,
                        &after[name],
                        arena::Arena {
                            objects: &objects,
                            extras: &extra_slots,
                        },
                        arena::Arena {
                            objects: &self.argument_objects,
                            extras: &self.argument_extra_slots,
                        },
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
            merge::objects(&mut joined, &self.argument_objects);
            extras::join(&mut joined_extras, &self.argument_extra_slots, &objects);
            module_states.join(&self.scopes, &self.modules, scopes.len());
            if let Some(joined) = &mut joined_scopes {
                bindings::join(joined, &self.scopes, &scopes);
            } else {
                joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
            }
            self.scopes[..scopes.len()].clone_from_slice(joined_scopes.as_ref().unwrap());
            for (id, slots) in &joined {
                self.argument_objects.insert(*id, slots.clone());
            }
            module_states.restore(&mut self.scopes, &self.modules);
            self.argument_extra_slots
                .clone_from(joined_extras.as_ref().unwrap());
            frames::compact(
                &mut self.scopes,
                frames::ModuleRoots {
                    original: scopes.len(),
                    modules: &mut self.modules,
                },
                &mut returned,
                &mut self.mapped_arguments,
                arena::ArenaMut {
                    objects: &mut self.argument_objects,
                    extras: &mut self.argument_extra_slots,
                },
                &mut self.fresh_mapped_parameters,
                &mut self.captured_bindings,
            );
            modules.clone_from(&self.modules);
            module_states.remapped(&self.scopes, &modules);
            joined_extras = Some(self.argument_extra_slots.clone());
            self.rebuild_captured_readers();
            self.rebuild_mapped_argument_owners();
            joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
            joined.clone_from(&self.argument_objects);
        }
        self.scopes[..scopes.len()].clone_from_slice(&scopes);
        self.modules = modules;
        module_states.restore(&mut self.scopes, &self.modules);
        self.argument_extra_slots = joined_extras.unwrap_or(extra_slots);
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
        self.install_disconnected_slots(&disconnected, disconnected_common, private_disconnected);
        self.definite_deleted_argument_slots = final_definite;
        self.refresh_captured_bindings();
        // Speculative identities with no surviving aliases cannot affect later reads.
        self.prune_alternative_state(&returned);
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        Value::Aggregate(returned)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod deletion_tests;

#[cfg(test)]
mod prefix_tests;
