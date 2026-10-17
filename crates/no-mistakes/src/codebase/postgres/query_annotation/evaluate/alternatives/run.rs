use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashSet;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let scopes = self.scopes.clone();
        let seen = self.active_callback_functions.clone();
        let executions = self.active_callback_executions.clone();
        let mut joined_seen = seen.clone();
        let captured = self.captured_bindings.clone();
        let mut joined_scopes: Option<Vec<Scope>> = None;
        let mut modules = self.modules.clone();
        self.begin_alternative_module_snapshot(scopes.len());
        let mut module_states = super::modules::States::default();
        let extra_slots = self.argument_extra_slots.clone();
        let mut joined_extras = None;
        let updates = self.builder_updates.clone();
        let untrusted = self.untrusted_array_intrinsics;
        let mut joined_untrusted = untrusted;
        let original_ids = self.next_builder;
        let original = self.invalidated_builders.clone();
        let objects = self.argument_objects.clone();
        let definite = self.definite_deleted_argument_slots.clone();
        let (baselines, mut joined_updates) = (self.alternative_builder_baselines(path), None);
        let recreated = self.recreated_argument_slots.clone();
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
        let mut final_live = None;
        let mut changed = FxHashSet::default();
        for (arm_index, arm) in arms.iter().enumerate() {
            self.untrusted_array_intrinsics = untrusted;
            // Keep arm-created frames alive for callbacks returned from helpers.
            self.scopes[..scopes.len()].clone_from_slice(&scopes);
            self.active_callback_functions.clone_from(&seen);
            self.active_callback_executions.clone_from(&executions);
            self.modules.clone_from(&modules);
            self.argument_extra_slots
                .retain(|id, _| !objects.contains_key(id));
            self.argument_extra_slots.extend(extra_slots.clone());
            self.restore_builder_updates(None, updates.clone(), original_ids);
            self.remove_alternative_captured_readers(scopes.len());
            self.captured_bindings
                .retain(|frame, _| *frame >= scopes.len());
            self.captured_bindings.extend(captured.clone());
            self.invalidated_builders.clone_from(&original);
            self.deleted_argument_slots.clone_from(&deleted);
            self.definite_deleted_argument_slots.clone_from(&definite);
            self.recreated_argument_slots.clone_from(&recreated);
            self.disconnected_argument_slots.clone_from(&disconnected);
            super::freshness::restore(&mut self.fresh_mapped_parameters, scopes.len(), &fresh);
            for (id, slots) in &objects {
                self.argument_objects.insert(*id, slots.clone());
            }
            self.restore_alternative_modules();
            // Existing frames retain their capture origins. Only frames made by
            // an arm (including restored module initials) need reader updates.
            self.add_alternative_captured_readers(scopes.len());
            self.rebuild_compacted_binding_indexes(false);
            returned.push(self.expr(arm, path, env, depth, generic));
            self.join_builder_updates(&mut joined_updates, &baselines, original_ids, path);
            joined_untrusted |= self.untrusted_array_intrinsics;
            deleted_changed.extend(self.deleted_argument_slots.iter().copied());
            changed.extend(self.invalidated_builders.iter().copied());
            self.merge_alternative_masks(
                &objects,
                &mut definite_common,
                &mut private_definite,
                &mut disconnected_common,
                &mut private_disconnected,
            );
            for (before, after) in scopes.iter().zip(&self.scopes) {
                for (name, value) in before {
                    // An arm can drop a name the other branch still has.
                    if !self.files[path].facts.precise_records {
                        super::values::changes(
                            value,
                            super::bindings::binding_or_unproven(after, name),
                            super::arena::Arena {
                                objects: &objects,
                                extras: &extra_slots,
                            },
                            super::arena::Arena {
                                objects: &self.argument_objects,
                                extras: &self.argument_extra_slots,
                            },
                            &mut changed,
                            &self.definite_deleted_argument_slots,
                        );
                    }
                }
            }
            super::freshness::unchanged(
                &mut fresh_joined,
                &self.fresh_mapped_parameters,
                &scopes,
                &self.scopes,
            );
            super::merge::objects(&mut joined, &self.argument_objects);
            super::extras::join(&mut joined_extras, &self.argument_extra_slots, &objects);
            module_states.join(
                &self.scopes,
                &self.modules,
                scopes.len(),
                self.files[path].facts.precise_records,
            );
            if let Some(joined) = &mut joined_scopes {
                super::bindings::join_with_sql(
                    joined,
                    &self.scopes,
                    &scopes,
                    self.files[path].facts.precise_records,
                );
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
            self.accumulate_callback_seen(&mut joined_seen, arm_index == 0);
            let indices = self.compact_alternative_frames(scopes.len(), &mut returned);
            self.remap_active_callbacks(scopes.len(), &indices.indices);
            super::callback_seen::remap(&mut joined_seen, scopes.len(), &indices.indices);
            modules.clone_from(&self.modules);
            module_states.remapped(&self.scopes, &modules);
            joined_extras = Some(self.argument_extra_slots.clone());
            self.remap_compacted_binding_indexes(
                scopes.len(),
                &indices.indices,
                indices.captured_changed,
            );
            joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
            joined.clone_from(&self.argument_objects);
            if arm_index + 1 == arms.len()
                && self.can_reuse_final_live(
                    scopes.len(),
                    captured.is_empty(),
                    original.is_empty() && changed.is_empty(),
                    &indices.indices,
                )
            {
                final_live = Some((indices.arguments, indices.identities));
            }
        }
        // Keep the final snapshot moves in this lexical scope and their original order.
        // Splitting their text adds no context struct, clones, or retained evaluator fields.
        include!("run_finish.inc")
    }
}
