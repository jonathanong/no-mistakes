use super::{Evaluator, Value};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

mod checkpoint;
mod counters;
mod entrypoints;
mod reachability;
mod tasks;

use {checkpoint::Checkpoint, counters::Counters, tasks::Task};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub fn run(&mut self, path: &Path) {
        let files = self.files;
        let file = &files[path];
        self.module_environment(path);
        self.prune_snapshot_state();
        let env = self.modules[path];
        let mut counts = Counters::new();
        let mut tasks = tasks::Tasks::new(file);
        let Some(first) = tasks.next() else {
            return;
        };
        let Some(second) = tasks.next() else {
            counts.plan(1);
            // Release any precise selector/DFS state before the only entrypoint.
            drop(tasks);
            self.rebuild_fresh_mapped_argument_bindings();
            operations::fresh();
            counts.fresh_rebuild();
            counts.task();
            self.run_task(first, path, env);
            return;
        };
        counts.plan(2);
        let checkpoint = Checkpoint::capture(self);
        counts.capture();
        // The first task starts from exactly the state just captured. Preserve
        // the historical freshness normalization without a redundant restore.
        self.rebuild_fresh_mapped_argument_bindings();
        operations::fresh();
        counts.fresh_rebuild();
        counts.task();
        self.run_task(first, path, env);
        checkpoint.restore(self);
        counts.restore();
        counts.fresh_rebuild();
        counts.task();
        self.run_task(second, path, env);
        for task in tasks {
            checkpoint.restore(self);
            counts.restore();
            counts.fresh_rebuild();
            counts.task();
            self.run_task(task, path, env);
        }
    }

    fn run_task(&mut self, task: Task<'_>, path: &Path, module_env: super::Environment) {
        match task {
            Task::Call(call) => {
                self.expr(call, path, &module_env, 16, false);
            }
            Task::Function(function) => {
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

#[cfg(test)]
#[path = "run/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "run/reference_tests.rs"]
mod reference;
#[cfg(test)]
#[path = "run/state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "run/operation_tests.rs"]
mod operations;

#[cfg(test)]
#[path = "run/imported_state_tests.rs"]
mod imported_state_tests;
#[cfg(not(test))]
#[path = "run/operations.rs"]
mod operations;
#[cfg(test)]
#[path = "run/state_test_support.rs"]
mod state_test_support;
