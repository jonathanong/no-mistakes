use super::{Environment, Evaluator, Value};
use crate::{codebase::postgres::query_annotation::Expr, fx::fx_map};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn module_environment(&mut self, path: &Path) -> Environment {
        if let Some(env) = self.modules.get(path) {
            return *env;
        }
        let mut env = self.environment(fx_map());
        // Publish before initialization: cycles and TDZ references share the
        // reserved module bindings instead of recursively initializing again.
        self.modules.insert(path.to_path_buf(), env);
        let facts = &self.files[path].facts;
        let roots = facts.roots.clone();
        let globals = facts.globals.clone();
        self.steps(&roots, path, &mut env, 16, true);
        // Unmodeled top-level statements must not discard independently
        // supported function bodies. Their captured values remain conservative.
        for (name, expr) in globals {
            match expr {
                Expr::Function(function) => {
                    self.scopes[env].insert(
                        name,
                        Value::Function(std::sync::Arc::new(function), path.to_path_buf(), env),
                    );
                }
                Expr::Unknown => {
                    self.scopes[env].insert(name, Value::Unknown);
                }
                _ => {}
            }
        }
        self.record_alternative_module_initial(path, env);
        env
    }
}
