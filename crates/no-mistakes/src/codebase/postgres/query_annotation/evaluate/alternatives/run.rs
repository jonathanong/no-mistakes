use super::super::{Environment, Evaluator, Value};
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
        let mut joined_scopes: Option<Vec<crate::fx::FxHashMap<String, Value>>> = None;
        let mut modules = self.modules.clone();
        self.begin_alternative_module_snapshot(scopes.len());
        let mut module_states = super::modules::States::default();
        let extra_slots = self.argument_extra_slots.clone();
        let mut joined_extras = None;
        let updates = self.builder_updates.clone();
        let original_ids = self.next_builder;
        let original = self.invalidated_builders.clone();
        let objects = self.argument_objects.clone();
        let definite = self.definite_deleted_argument_slots.clone();
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
        let mut changed = FxHashSet::default();
        for (arm_index, arm) in arms.iter().enumerate() {
            // Keep arm-created frames alive for callbacks returned from helpers.
            self.scopes[..scopes.len()].clone_from_slice(&scopes);
            self.active_callback_functions.clone_from(&seen);
            self.active_callback_executions.clone_from(&executions);
            self.modules.clone_from(&modules);
            self.argument_extra_slots
                .retain(|id, _| !objects.contains_key(id));
            self.argument_extra_slots.extend(extra_slots.clone());
            self.builder_updates.retain(|id, _| *id >= original_ids);
            self.builder_updates.extend(updates.clone());
            self.captured_bindings
                .retain(|frame, _| *frame >= scopes.len());
            self.captured_bindings.extend(captured.clone());
            self.rebuild_binding_indexes();
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
            self.rebuild_binding_indexes();
            returned.push(self.expr(arm, path, env, depth, generic));
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
            super::freshness::unchanged(
                &mut fresh_joined,
                &self.fresh_mapped_parameters,
                &scopes,
                &self.scopes,
            );
            super::merge::objects(&mut joined, &self.argument_objects);
            super::extras::join(&mut joined_extras, &self.argument_extra_slots, &objects);
            module_states.join(&self.scopes, &self.modules, scopes.len());
            if let Some(joined) = &mut joined_scopes {
                super::bindings::join(joined, &self.scopes, &scopes);
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
            let indices = super::frames::compact(
                &mut self.scopes,
                super::frames::ModuleRoots {
                    original: scopes.len(),
                    modules: &mut self.modules,
                    initials: &mut self.active_module_initials,
                },
                &mut returned,
                &mut self.mapped_arguments,
                super::arena::ArenaMut {
                    objects: &mut self.argument_objects,
                    extras: &mut self.argument_extra_slots,
                },
                &mut self.fresh_mapped_parameters,
                &mut self.captured_bindings,
            );
            self.remap_active_callbacks(scopes.len(), &indices);
            super::callback_seen::remap(&mut joined_seen, scopes.len(), &indices);
            modules.clone_from(&self.modules);
            module_states.remapped(&self.scopes, &modules);
            joined_extras = Some(self.argument_extra_slots.clone());
            self.rebuild_binding_indexes();
            joined_scopes = Some(self.scopes[..scopes.len()].to_vec());
            joined.clone_from(&self.argument_objects);
        }
        self.scopes[..scopes.len()].clone_from_slice(&scopes);
        self.active_callback_functions = joined_seen;
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
        super::freshness::restore(
            &mut self.fresh_mapped_parameters,
            scopes.len(),
            &fresh_joined,
        );
        self.rebuild_fresh_mapped_argument_bindings();
        self.invalidated_builders = original;
        self.deleted_argument_slots = deleted_changed;
        // Recreation inside one arm must not make a deleted property in
        // another arm look live. A write after the branch records its own.
        self.recreated_argument_slots = recreated;
        let mut final_definite = definite_common.unwrap_or(definite.clone());
        final_definite.extend(private_definite);
        // These identities already represent actual arm effects. Replaying an
        // arguments escape would wrongly taint values assigned after that escape.
        self.invalidated_builders.extend(changed.iter().copied());
        super::values::apply_taint(&mut self.scopes, &changed);
        // Restored bindings must retain any prior opaque parameter update before
        // the common deletion disconnects them from future argument mutations.
        self.install_disconnected_slots(&disconnected, disconnected_common, private_disconnected);
        self.definite_deleted_argument_slots = final_definite;
        self.refresh_captured_bindings();
        // Speculative identities with no surviving aliases cannot affect later reads.
        self.active_module_initials
            .pop()
            .expect("balanced alternatives");
        self.prune_alternative_state(&returned);
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        Value::Aggregate(returned)
    }
}
