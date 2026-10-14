use super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub fn run(&mut self, path: &Path) {
        let file = &self.files[path];
        let globals = &file.facts.globals;
        self.module_environment(path);
        self.prune_snapshot_state();
        let env = self.modules[path];
        let unmodeled = &file.facts.unmodeled_calls;
        let scopes = self.scopes.clone();
        let modules = self.modules.clone();
        let invalidated = self.invalidated_builders.clone();
        let deleted = self.deleted_argument_slots.clone();
        let mapped = self.mapped_arguments.clone();
        let fresh = self.fresh_mapped_parameters.clone();
        let objects = self.argument_objects.clone();
        let extra_slots = self.argument_extra_slots.clone();
        let definite = self.definite_deleted_argument_slots.clone();
        let recreated = self.recreated_argument_slots.clone();
        let disconnected = self.disconnected_argument_slots.clone();
        let builder_updates = self.builder_updates.clone();
        let captured_bindings = self.captured_bindings.clone();
        let captured_binding_readers = self.captured_binding_readers.clone();
        let mapped_argument_owners = self.mapped_argument_owners.clone();
        let mapped_parameter_indices = self.mapped_parameter_indices.clone();
        for call in unmodeled {
            if matches!(call, Expr::Call { start, end, .. } if !file.executors.contains(&(*start, *end)))
            {
                self.scopes.clone_from(&scopes);
                self.modules.clone_from(&modules);
                self.invalidated_builders.clone_from(&invalidated);
                self.deleted_argument_slots.clone_from(&deleted);
                self.mapped_arguments.clone_from(&mapped);
                self.fresh_mapped_parameters.clone_from(&fresh);
                self.rebuild_fresh_mapped_argument_bindings();
                self.argument_objects.clone_from(&objects);
                self.argument_extra_slots.clone_from(&extra_slots);
                self.definite_deleted_argument_slots.clone_from(&definite);
                self.recreated_argument_slots.clone_from(&recreated);
                self.disconnected_argument_slots.clone_from(&disconnected);
                self.builder_updates.clone_from(&builder_updates);
                self.captured_bindings.clone_from(&captured_bindings);
                self.captured_binding_readers
                    .clone_from(&captured_binding_readers);
                self.mapped_argument_owners
                    .clone_from(&mapped_argument_owners);
                self.mapped_parameter_indices
                    .clone_from(&mapped_parameter_indices);
                self.expr(call, path, &env, 16, false);
            }
        }
        // Function declarations describe possible entrypoints; contextual
        // callback invocations take precedence over this unknown input.
        for (name, expr) in globals {
            // Only exported functions are independently callable by a
            // supplied-source consumer. A private callback still contributes
            // its contextual invocations, but speculative calls to it must
            // not erase the exact argument supplied by its owner.
            if file.facts.precise_records
                && !file.module.ts.exported_bindings.iter().any(|binding| {
                    binding.specifier.is_none()
                        && exported_reaches(globals, &binding.local, name, &mut BTreeSet::new())
                })
            {
                continue;
            }
            if let Expr::Function(function) = expr {
                // Speculative entrypoints share initialized facts, never effects
                // from an unrelated function considered earlier in name order.
                self.scopes.clone_from(&scopes);
                self.modules.clone_from(&modules);
                self.invalidated_builders.clone_from(&invalidated);
                self.deleted_argument_slots.clone_from(&deleted);
                self.mapped_arguments.clone_from(&mapped);
                self.fresh_mapped_parameters.clone_from(&fresh);
                self.rebuild_fresh_mapped_argument_bindings();
                self.argument_objects.clone_from(&objects);
                self.argument_extra_slots.clone_from(&extra_slots);
                self.definite_deleted_argument_slots.clone_from(&definite);
                self.recreated_argument_slots.clone_from(&recreated);
                self.disconnected_argument_slots.clone_from(&disconnected);
                self.builder_updates.clone_from(&builder_updates);
                self.captured_bindings.clone_from(&captured_bindings);
                self.captured_binding_readers
                    .clone_from(&captured_binding_readers);
                self.mapped_argument_owners
                    .clone_from(&mapped_argument_owners);
                self.mapped_parameter_indices
                    .clone_from(&mapped_parameter_indices);
                let mut values: FxHashMap<String, Value> = function
                    .params
                    .iter()
                    .map(|name| (name.clone(), Value::Unknown))
                    .collect();
                if !function.arrow {
                    values.insert("arguments".into(), Value::Unknown);
                }
                if let Some(name) = &function.self_name {
                    values.insert(name.clone(), Value::Unknown);
                }
                let mut env = self.environment(values);
                self.steps(&function.body, path, &mut env, 16, true);
            }
        }
    }
}

fn exported_reaches(
    globals: &BTreeMap<String, Expr>,
    exported: &str,
    target: &str,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if exported == target {
        return true;
    }
    if !visiting.insert(exported.to_string()) {
        return false;
    }
    let reaches = globals
        .get(exported)
        .is_some_and(|value| value_reaches(globals, value, target, visiting));
    visiting.remove(exported);
    reaches
}

fn value_reaches(
    globals: &BTreeMap<String, Expr>,
    value: &Expr,
    target: &str,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match value {
        Expr::Name(name) => exported_reaches(globals, name, target, visiting),
        Expr::Record(values, _) | Expr::Object(values) | Expr::Container(values) => values
            .iter()
            .any(|value| value_reaches(globals, value, target, visiting)),
        Expr::Function(function) => function.body.iter().any(|step| {
            matches!(step, super::super::Step::Return(value) if value_reaches(globals, value, target, visiting))
        }),
        _ => false,
    }
}

#[cfg(test)]
#[path = "run/tests.rs"]
mod tests;
