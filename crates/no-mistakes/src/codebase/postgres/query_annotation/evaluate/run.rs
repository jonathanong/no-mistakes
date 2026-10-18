use super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod entrypoints;
mod reachability;

use reachability::export_reachable_globals;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub fn run(&mut self, path: &Path) {
        let file = &self.files[path];
        let globals = &file.facts.globals;
        self.module_environment(path);
        self.prune_snapshot_state();
        let env = self.modules[path];
        let unmodeled = &file.facts.unmodeled_calls;
        for call in unmodeled {
            if file.facts.precise_records
                && matches!(call, Expr::Call { start, end, .. } if file.facts.branch_calls.contains(&(*start, *end)))
            {
                continue;
            }
            if matches!(call, Expr::Call { start, end, .. } if !file.executors.contains(&(*start, *end)))
            {
                let checkpoint = self.begin_root_transaction();
                self.expr(call, path, &env, 16, false);
                self.finish_root_transaction(checkpoint);
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
                entrypoints::collect(
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
                let checkpoint = self.begin_root_transaction();
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
                self.finish_root_transaction(checkpoint);
            }
        }
    }
}

#[cfg(test)]
#[path = "run/tests.rs"]
mod tests;
