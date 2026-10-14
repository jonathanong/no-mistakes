use super::{Environment, Evaluator, Value};
use crate::{
    codebase::{
        dependencies::extract::ImportKind,
        postgres::query_annotation::{Expr, Step},
    },
    fx::fx_map,
};
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
        let files = self.files;
        if files[path].facts.precise_records {
            let module = &files[path].module;
            let mut unresolved_relative = false;
            let eager_targets = module
                .ts
                .imports
                .iter()
                .filter(|import| {
                    import.kind == ImportKind::Static && import.function_scope.is_none()
                })
                .filter_map(|import| {
                    let target = module.target(&import.specifier, path, &self.resolve);
                    if target.is_none() && import.specifier.starts_with('.') {
                        unresolved_relative = true;
                    }
                    target.map(Path::to_path_buf)
                })
                .collect::<Vec<_>>();
            for target in eager_targets {
                self.module_environment(&target);
            }
            // Initialize every supplied dependency before invalidating. A
            // missing eager module could itself import and mutate any of them.
            if unresolved_relative {
                self.invalidate_live_builders();
            }
        }
        let facts = &files[path].facts;
        self.steps(&facts.roots, path, &mut env, 16, true);
        // A dependency's top-level opaque effect may mutate a builder owned
        // by another already-initialized module. Its local frame cannot prove
        // that shared builder unchanged.
        if facts.precise_records
            && facts.roots.iter().any(|step| match step {
                Step::Bind(_, expr) | Step::Effect(expr) => self.effect_can_mutate(expr, path, env),
                Step::Append(_, _) | Step::Unsupported | Step::PotentialCalls(_) => true,
                Step::Reserve(_) | Step::Hoisted(_, _) | Step::Var(_) | Step::Return(_) => false,
            })
        {
            self.invalidate_live_builders();
        }
        // Unmodeled top-level statements must not discard independently
        // supported function bodies. Their captured values remain conservative.
        for (name, expr) in &facts.globals {
            match expr {
                Expr::Function(function) => {
                    self.scopes[env].insert(
                        name.clone(),
                        Value::Function(std::sync::Arc::clone(function), path.to_path_buf(), env),
                    );
                }
                Expr::Unknown => {
                    self.scopes[env].insert(name.clone(), Value::Unknown);
                }
                _ => {}
            }
        }
        self.record_alternative_module_initial(path, env);
        env
    }
}
