// Frozen exact9651 run algorithm. Test-only differential oracle.
use super::super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::codebase::postgres::query_annotation::Function;
use crate::fx::FxHashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::reachability::export_reachable_globals;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn run_reference(&mut self, path: &Path) {
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
        let untrusted_array_intrinsics = self.untrusted_array_intrinsics;
        let captured_bindings = self.captured_bindings.clone();
        let captured_binding_readers = self.captured_binding_readers.clone();
        let mapped_argument_owners = self.mapped_argument_owners.clone();
        let mapped_parameter_indices = self.mapped_parameter_indices.clone();
        super::operations::capture();
        for call in unmodeled {
            if file.facts.precise_records
                && matches!(call, Expr::Call { start, end, .. } if file.facts.branch_calls.contains(&(*start, *end)))
            {
                continue;
            }
            if matches!(call, Expr::Call { start, end, .. } if !file.executors.contains(&(*start, *end)))
            {
                super::operations::restore();
                self.scopes.clone_from(&scopes);
                self.modules.clone_from(&modules);
                self.invalidated_builders.clone_from(&invalidated);
                self.deleted_argument_slots.clone_from(&deleted);
                self.mapped_arguments.clone_from(&mapped);
                self.fresh_mapped_parameters.clone_from(&fresh);
                self.rebuild_fresh_mapped_argument_bindings();
                super::operations::fresh();
                self.argument_objects.clone_from(&objects);
                self.argument_extra_slots.clone_from(&extra_slots);
                self.definite_deleted_argument_slots.clone_from(&definite);
                self.recreated_argument_slots.clone_from(&recreated);
                self.disconnected_argument_slots.clone_from(&disconnected);
                self.builder_updates.clone_from(&builder_updates);
                self.untrusted_array_intrinsics = untrusted_array_intrinsics;
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
        let reachable = file.facts.precise_records.then(|| {
            export_reachable_globals(
                globals,
                file.module
                    .ts
                    .exported_bindings
                    .iter()
                    .filter(|binding| binding.specifier.is_none())
                    .map(|binding| binding.local.as_str()),
            )
        });
        let mut seen_functions = BTreeSet::new();
        for (name, expr) in globals {
            // Only exported functions are independently callable by a
            // supplied-source consumer. A private callback still contributes
            // its contextual invocations, but speculative calls to it must
            // not erase the exact argument supplied by its owner.
            if reachable
                .as_ref()
                .is_some_and(|names| !names.contains(name.as_str()))
            {
                continue;
            }
            let mut functions = Vec::new();
            if file.facts.precise_records {
                collect(
                    globals,
                    expr,
                    &mut BTreeSet::new(),
                    &mut seen_functions,
                    &mut functions,
                );
            } else if let Expr::Function(function) = expr {
                functions.push(std::sync::Arc::clone(function));
            }
            for function in functions {
                // Speculative entrypoints share initialized facts, never effects
                // from an unrelated function considered earlier in name order.
                super::operations::restore();
                self.scopes.clone_from(&scopes);
                self.modules.clone_from(&modules);
                self.invalidated_builders.clone_from(&invalidated);
                self.deleted_argument_slots.clone_from(&deleted);
                self.mapped_arguments.clone_from(&mapped);
                self.fresh_mapped_parameters.clone_from(&fresh);
                self.rebuild_fresh_mapped_argument_bindings();
                super::operations::fresh();
                self.argument_objects.clone_from(&objects);
                self.argument_extra_slots.clone_from(&extra_slots);
                self.definite_deleted_argument_slots.clone_from(&definite);
                self.recreated_argument_slots.clone_from(&recreated);
                self.disconnected_argument_slots.clone_from(&disconnected);
                self.builder_updates.clone_from(&builder_updates);
                self.untrusted_array_intrinsics = untrusted_array_intrinsics;
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

// Frozen recursive planner, kept in test-only source alongside the old run.
pub(super) fn collect(
    globals: &BTreeMap<String, Expr>,
    value: &Expr,
    visiting: &mut BTreeSet<String>,
    seen: &mut BTreeSet<u32>,
    functions: &mut Vec<Arc<Function>>,
) {
    match value {
        Expr::Function(function) => {
            if seen.insert(function.start) {
                functions.push(Arc::clone(function));
            }
        }
        Expr::Name(name) => {
            if visiting.insert(name.clone()) {
                if let Some(value) = globals.get(name) {
                    collect(globals, value, visiting, seen, functions);
                }
                visiting.remove(name);
            }
        }
        Expr::Record(values, _)
        | Expr::Object(values)
        | Expr::Array(values)
        | Expr::Container(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values) => {
            for value in values {
                collect(globals, value, visiting, seen, functions);
            }
        }
        Expr::Spread(value)
        | Expr::Member(value, _)
        | Expr::Index(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => collect(globals, value, visiting, seen, functions),
        Expr::ComputedMember(base, key) | Expr::Append(base, key) => {
            collect(globals, base, visiting, seen, functions);
            collect(globals, key, visiting, seen, functions);
        }
        _ => {}
    }
}
