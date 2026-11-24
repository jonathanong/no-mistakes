use super::{Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Function};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod entrypoints;
mod preload;
mod reachability;
mod snapshot;

use reachability::export_reachable_globals;
use snapshot::RunSnapshot;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub fn run(&mut self, path: &Path) {
        let file = &self.files[path];
        let globals = &file.facts.globals;
        self.module_environment(path);
        super::super::progress::trace("postgres.annotation.snapshot", path, None, || {
            self.prune_snapshot_state()
        });
        let env = self.modules[&self.module_ids().id(path)];
        let runnable_unmodeled = file
            .facts
            .unmodeled_calls
            .iter()
            .filter(|call| {
                !(file.facts.precise_records
                    && matches!(call, Expr::Call { start, end, .. } if file.facts.branch_calls.contains(&(*start, *end))))
                    && matches!(call, Expr::Call { start, end, .. } if !file.executors.contains(&(*start, *end)))
            })
            .collect::<Vec<_>>();
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
        let mut runnable_functions = Vec::new();
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
            if file.facts.precise_records {
                entrypoints::collect(
                    globals,
                    expr,
                    &mut BTreeSet::new(),
                    &mut seen_functions,
                    &mut runnable_functions,
                );
            } else if let Expr::Function(function) = expr {
                runnable_functions.push(Arc::clone(function));
            }
        }
        let total_tasks = runnable_unmodeled.len() + runnable_functions.len();
        if total_tasks == 0 {
            return;
        }
        if total_tasks == 1 {
            if let Some(call) = runnable_unmodeled.first() {
                let timing = super::super::progress::timing("postgres.annotation.unmodeled_call");
                self.expr(call, path, &env, 16, false);
                drop(timing);
            } else if let Some(function) = runnable_functions.first() {
                self.run_entrypoint(path, function);
            }
            return;
        }
        let mut snapshot = RunSnapshot::capture(self);
        let mut unpromotable = FxHashSet::default();
        preload::preload_referenced_imports(
            self,
            path,
            &runnable_unmodeled,
            &runnable_functions,
            &mut snapshot,
            &mut unpromotable,
        );
        let mut completed = 0usize;
        let mut needs_restore = false;
        for call in runnable_unmodeled {
            if needs_restore {
                snapshot.restore(self);
            }
            let timing = super::super::progress::timing("postgres.annotation.unmodeled_call");
            self.expr(call, path, &env, 16, false);
            drop(timing);
            completed += 1;
            if completed < total_tasks {
                needs_restore = !snapshot.promote_loaded(self, &mut unpromotable);
            }
        }
        for function in runnable_functions {
            // Speculative entrypoints share initialized facts, never effects
            // from an unrelated function considered earlier in name order.
            if needs_restore {
                snapshot.restore(self);
            }
            self.run_entrypoint(path, &function);
            completed += 1;
            if completed < total_tasks {
                needs_restore = !snapshot.promote_loaded(self, &mut unpromotable);
            }
        }
    }

    fn run_entrypoint(&mut self, path: &Path, function: &Function) {
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
        super::super::progress::trace(
            "postgres.annotation.entrypoint",
            path,
            Some(function.start),
            || self.steps(&function.body, path, &mut env, 16, true),
        );
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "run/tests.rs"]
mod tests;
