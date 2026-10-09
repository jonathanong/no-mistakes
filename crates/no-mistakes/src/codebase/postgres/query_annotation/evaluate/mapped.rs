mod bindings;
mod fresh;
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
        use crate::codebase::postgres::query_annotation::Step;
        let mut mappings = self
            .mapped_arguments
            .get(&captured)
            .cloned()
            .unwrap_or_default();
        for (_, params) in &mut mappings {
            for name in params {
                let shadowed = function.params.contains(name)
                    || function.self_name.as_ref() == Some(name)
                    || function.body.iter().any(|step| match step {
                        Step::Bind(local, _) | Step::Hoisted(local, _) | Step::Var(local) => {
                            local == name
                        }
                        Step::Reserve(names) => names.contains(name),
                        _ => false,
                    });
                if shadowed {
                    name.clear();
                }
            }
        }
        self.inherit_freshness(scope, captured, &mappings);
        if let Some(own) = own {
            mappings.push(own);
        }
        if !mappings.is_empty() {
            self.mapped_arguments.insert(scope, mappings);
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
                            && !self.definite_deleted_argument_slots.contains(&(*id, index))
                    })
            })
        })
    }

    pub(super) fn disconnect_mapped_slot(&mut self, id: u64, index: usize) {
        if !self.invalidated_builders.contains(&id)
            || self.definite_deleted_argument_slots.contains(&(id, index))
        {
            return;
        }
        // Disconnection stops future updates; it cannot undo an opaque update
        // that already reached the parameter through the invocation object.
        for (env, mappings) in &self.mapped_arguments {
            for (object, params) in mappings {
                if *object != id {
                    continue;
                }
                if let Some(name) = params.get(index).filter(|name| {
                    !name.is_empty()
                        && params.iter().rposition(|param| param == *name) == Some(index)
                }) {
                    if self
                        .fresh_mapped_parameters
                        .get(env)
                        .is_some_and(|names| names.contains(name))
                    {
                        continue;
                    }
                    let value = self.scopes[*env]
                        .get_mut(name)
                        .expect("mapped parameter binding");
                    let previous = std::mem::replace(value, Value::Unknown);
                    *value = match previous {
                        Value::Prefix(_, _, _) | Value::Unknown => Value::Unknown,
                        previous => Value::Aggregate(vec![Value::Unknown, previous]),
                    };
                }
            }
        }
    }

    pub(super) fn update_mapped_parameter(&mut self, env: Environment, name: &str, value: &Value) {
        let mut mapped = false;
        for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
            // With duplicate sloppy parameters, only the last occurrence maps.
            if let Some(index) = params.iter().rposition(|param| param == name) {
                mapped = true;
                if !self.definite_deleted_argument_slots.contains(&(*id, index)) {
                    // Callback and container values retain their modeled identities;
                    // opaque consumers must still see their captures and aliases.
                    self.argument_objects
                        .get_mut(id)
                        .expect("mapped arguments object")[index] = value.clone();
                }
            }
        }
        if mapped {
            self.fresh_mapped_parameters
                .entry(env)
                .or_default()
                .insert(name.to_string());
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod fresh_tests;

#[cfg(test)]
mod binding_tests;
