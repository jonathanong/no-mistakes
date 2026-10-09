mod bindings;
mod fresh;
mod fresh_index;
mod owners;
mod possible;
use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Function;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn parameter_mapping(
        &self,
        function: &Function,
        path: &Path,
        object: &Value,
    ) -> Option<(u64, Vec<String>)> {
        match object {
            Value::Arguments(id)
                if self.files[path]
                    .facts
                    .mapped_arguments
                    .contains(&function.start) =>
            {
                let mut params = function.params.clone();
                for name in &mut params {
                    if function
                        .params
                        .iter()
                        .rposition(|param| param == name)
                        .is_some_and(|index| index >= self.argument_objects[id].len())
                    {
                        name.clear();
                    }
                }
                Some((*id, params))
            }
            _ => None,
        }
    }

    pub(super) fn register_mappings(
        &mut self,
        scope: Environment,
        captured: Environment,
        function: &Function,
        own: Option<(u64, Vec<String>)>,
    ) {
        let mut mappings = self
            .mapped_arguments
            .get(&captured)
            .cloned()
            .unwrap_or_default();
        for (_, params) in &mut mappings {
            for name in params {
                if super::calls::scopes::shadows(function, name) {
                    name.clear();
                }
            }
        }
        self.inherit_freshness(scope, captured, &mappings);
        if let Some(own) = own {
            self.mapped_argument_owners.insert(own.0, scope);
            mappings.push(own);
        }
        if !mappings.is_empty() {
            self.mapped_arguments.insert(scope, mappings);
            self.index_mapped_freshness(scope);
        }
    }

    pub(super) fn write_mapped_argument_slot(&mut self, id: u64, index: usize, value: &Value) {
        if let Some((frame, name)) = self.mapped_slot_target(id, index) {
            self.write_captured_binding(frame, &name, value);
        }
    }

    pub(super) fn mapped_parameter_unknown(&self, env: Environment, name: &str) -> bool {
        if self
            .fresh_mapped_parameters
            .get(&env)
            .is_some_and(|names| names.contains(name))
        {
            return false;
        }
        self.mapped_arguments.get(&env).is_some_and(|mappings| {
            mappings.iter().any(|(id, params)| {
                params
                    .iter()
                    .rposition(|param| param == name)
                    .is_some_and(|index| {
                        self.invalidated_builders.contains(id)
                            && !self.disconnected_argument_slots.contains(&(*id, index))
                    })
            })
        })
    }

    pub(super) fn disconnect_mapped_slot(&mut self, id: u64, index: usize) {
        if !self.invalidated_builders.contains(&id)
            || self.disconnected_argument_slots.contains(&(id, index))
        {
            return;
        }
        // Disconnection stops future updates; it cannot undo an opaque update
        // that already reached the parameter through the invocation object.
        let Some((owner, name)) = self.mapped_slot_target(id, index) else {
            return;
        };
        for frame in self.captured_write_targets(owner, &name) {
            if self
                .fresh_mapped_parameters
                .get(&frame)
                .is_some_and(|names| names.contains(&name))
            {
                continue;
            }
            // Read the live property before deletion clears it. Opaque escape
            // leaves uncertainty, but cannot erase a later known candidate.
            let value = self
                .mapped_parameter_value(frame, &name)
                .expect("dirty connected mapped parameter");
            self.scopes[frame].insert(name.clone(), value);
        }
    }

    pub(super) fn update_mapped_parameter(&mut self, env: Environment, name: &str, value: &Value) {
        if self.sync_mapped_parameter(env, name, value).0 {
            self.mark_mapped_fresh(env, name);
        }
    }
    pub(super) fn sync_mapped_parameter(
        &mut self,
        env: Environment,
        name: &str,
        value: &Value,
    ) -> (bool, bool) {
        let mut mapped = false;
        let mut escaped = false;
        for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
            // With duplicate sloppy parameters, only the last occurrence maps.
            if let Some(index) = params.iter().rposition(|param| param == name) {
                mapped = true;
                if !self.disconnected_argument_slots.contains(&(*id, index)) {
                    escaped |= self.invalidated_builders.contains(id);
                    // Callback and container values retain their modeled identities;
                    // opaque consumers must still see their captures and aliases.
                    self.argument_objects
                        .get_mut(id)
                        .expect("mapped arguments object")[index] = value.clone();
                }
            }
        }
        (mapped, escaped)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod fresh_tests;

#[cfg(test)]
mod binding_tests;
