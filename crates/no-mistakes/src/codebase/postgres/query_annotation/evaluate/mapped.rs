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
            Value::Arguments(id, values)
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
                        .is_some_and(|index| index >= values.len())
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
        if let Some(own) = own {
            mappings.push(own);
        }
        if !mappings.is_empty() {
            self.mapped_arguments.insert(scope, mappings);
        }
    }

    pub(super) fn mapped_parameter_unknown(&self, env: Environment, name: &str) -> bool {
        self.mapped_arguments.get(&env).is_some_and(|mappings| {
            mappings.iter().any(|(id, params)| {
                params
                    .iter()
                    .rposition(|param| param == name)
                    .is_some_and(|index| {
                        self.invalidated_builders.contains(id)
                            && !self.deleted_argument_slots.contains(&(*id, Some(index)))
                    })
            })
        })
    }

    pub(super) fn update_mapped_parameter(&mut self, env: Environment, name: &str, value: &Value) {
        for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
            // With duplicate sloppy parameters, only the last occurrence maps.
            if let Some(index) = params.iter().rposition(|param| param == name) {
                if !self.deleted_argument_slots.contains(&(*id, Some(index))) {
                    // Callback and container values retain their modeled identities;
                    // opaque consumers must still see their captures and aliases.
                    let replacement = value.clone();
                    for scope in &mut self.scopes {
                        for value in scope.values_mut() {
                            update_slot(value, *id, index, &replacement);
                        }
                    }
                }
            }
        }
    }
}

fn update_slot(value: &mut Value, id: u64, index: usize, replacement: &Value) {
    match value {
        Value::Arguments(found, values) if *found == id => {
            // Mapping registration includes only supplied slots; container
            // values keep their length when a slot is deleted or invalidated.
            values[index] = replacement.clone();
        }
        Value::Arguments(_, values) | Value::Aggregate(values) => {
            for value in values {
                update_slot(value, id, index, replacement);
            }
        }
        Value::Promise(value) => update_slot(value, id, index, replacement),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
