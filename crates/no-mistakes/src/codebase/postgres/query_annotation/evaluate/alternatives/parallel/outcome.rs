mod take;
use super::super::super::{ArgumentObjects, CapturedBindings, ExecutionEvents, Scope};
use super::translate::Translation;
use super::*;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
/// A serialization-ready private suffix: no resolver, catalog, modules or source ownership.
pub(in crate::codebase::postgres::query_annotation::evaluate) struct Outcome {
    base_env: usize,
    base_id: u64,
    value: Value,
    events: ExecutionEvents,
    scopes: Vec<Scope>,
    next_builder: u64,
    invalidated_builders: FxHashSet<u64>,
    builder_updates: FxHashMap<u64, Value>,
    captured_bindings: CapturedBindings,
    captured_binding_readers: FxHashMap<Environment, FxHashMap<String, FxHashSet<Environment>>>,
    mapped_argument_owners: FxHashMap<u64, Environment>,
    mapped_parameter_indices: FxHashMap<u64, FxHashMap<String, usize>>,
    deleted_argument_slots: FxHashSet<(u64, Option<usize>)>,
    argument_objects: ArgumentObjects,
    argument_extra_slots: FxHashMap<u64, BTreeMap<usize, Value>>,
    definite_deleted_argument_slots: FxHashSet<(u64, usize)>,
    recreated_argument_slots: FxHashSet<(u64, usize)>,
    disconnected_argument_slots: FxHashSet<(u64, usize)>,
    fresh_mapped_parameters: FxHashMap<Environment, FxHashSet<String>>,
    fresh_mapped_argument_bindings: FxHashMap<u64, FxHashMap<Environment, FxHashSet<String>>>,
    mapped_arguments: FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
}
impl Outcome {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn install<F>(
        self,
        evaluator: &mut Evaluator<'_, F>,
    ) -> Value {
        let mut translation = Translation::new(
            self.base_env,
            self.base_id,
            evaluator.scopes.len(),
            evaluator.next_builder,
        );
        let envs = (self.base_env..self.base_env + self.scopes.len())
            .map(|env| (env, translation.env(env)))
            .collect::<FxHashMap<_, _>>();
        evaluator.next_builder += self.next_builder - self.base_id;
        for mut scope in self.scopes {
            if !translation.identity() {
                let edits = scope
                    .iter()
                    .filter_map(|(name, value)| {
                        translation
                            .changed(value)
                            .map(|value| (name.clone(), value))
                    })
                    .collect::<Vec<_>>();
                if !edits.is_empty() {
                    scope.extend(edits);
                }
            }
            evaluator.scopes.push(scope);
        }
        for (id, mut slots) in self.argument_objects {
            if !translation.identity() {
                for index in 0..slots.len() {
                    if let Some(value) = translation.changed(&slots[index]) {
                        slots[index] = value;
                    }
                }
            }
            evaluator.argument_objects.insert(translation.id(id), slots);
        }
        for (id, slots) in self.argument_extra_slots {
            let slots = slots
                .into_iter()
                .map(|(index, value)| (index, translation.value(value)))
                .collect();
            evaluator
                .argument_extra_slots
                .insert(translation.id(id), slots);
        }
        for (env, mut origins) in self.captured_bindings {
            origins.remap(&envs);
            evaluator
                .captured_bindings
                .insert(translation.env(env), origins);
        }
        for (env, names) in self.captured_binding_readers {
            let entry = evaluator
                .captured_binding_readers
                .entry(translation.env(env))
                .or_default();
            for (name, readers) in names {
                entry
                    .entry(name)
                    .or_default()
                    .extend(readers.into_iter().map(|env| translation.env(env)));
            }
        }
        evaluator.invalidated_builders.extend(
            self.invalidated_builders
                .into_iter()
                .map(|id| translation.id(id)),
        );
        for (id, value) in self.builder_updates {
            let value = translation.value(value);
            evaluator.builder_updates.insert(translation.id(id), value);
        }
        evaluator.mapped_argument_owners.extend(
            self.mapped_argument_owners
                .into_iter()
                .map(|(id, env)| (translation.id(id), translation.env(env))),
        );
        evaluator.mapped_parameter_indices.extend(
            self.mapped_parameter_indices
                .into_iter()
                .map(|(id, names)| (translation.id(id), names)),
        );
        evaluator.fresh_mapped_parameters.extend(
            self.fresh_mapped_parameters
                .into_iter()
                .map(|(env, names)| (translation.env(env), names)),
        );
        for (id, bindings) in self.fresh_mapped_argument_bindings {
            let bindings = bindings
                .into_iter()
                .map(|(env, names)| (translation.env(env), names))
                .collect();
            evaluator
                .fresh_mapped_argument_bindings
                .insert(translation.id(id), bindings);
        }
        for (env, arguments) in self.mapped_arguments {
            let arguments = arguments
                .into_iter()
                .map(|(id, names)| (translation.id(id), names))
                .collect();
            evaluator
                .mapped_arguments
                .insert(translation.env(env), arguments);
        }
        evaluator.deleted_argument_slots.extend(
            self.deleted_argument_slots
                .into_iter()
                .map(|(id, slot)| (translation.id(id), slot)),
        );
        evaluator.definite_deleted_argument_slots.extend(
            self.definite_deleted_argument_slots
                .into_iter()
                .map(|(id, slot)| (translation.id(id), slot)),
        );
        evaluator.recreated_argument_slots.extend(
            self.recreated_argument_slots
                .into_iter()
                .map(|(id, slot)| (translation.id(id), slot)),
        );
        evaluator.disconnected_argument_slots.extend(
            self.disconnected_argument_slots
                .into_iter()
                .map(|(id, slot)| (translation.id(id), slot)),
        );
        // Preserve executor observations even if factory eligibility widens later.
        // Values can reference this arm's private environments and object IDs.
        for (call, values) in self.events {
            evaluator.events.entry(call).or_default().extend(
                values
                    .into_iter()
                    .map(|(flag, value)| (flag, translation.value(value))),
            );
        }
        translation.value(self.value)
    }
}
