use super::{Environment, Evaluator, File, Scope};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::{fx_map, FxHashMap};
use std::path::{Path, PathBuf};

fn count(observer: Option<&crate::diagnostics::InvocationObserver>, name: &'static str) {
    if let Some(observer) = observer {
        observer.increment(name, 1);
    }
}

fn independent(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Text(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Primitive | Expr::Function(_)
    )
}

fn eligibility(file: &File<'_>) -> Result<(), &'static str> {
    if file.facts.roots.is_empty() {
        return Err("postgres.annotation.initial_template.rejected_empty");
    }
    for step in &file.facts.roots {
        match step {
            Step::Reserve(_) | Step::Var(_) => {}
            Step::Bind(_, expr) | Step::Hoisted(_, expr) if independent(expr) => {}
            Step::Bind(_, _) | Step::Hoisted(_, _) => {
                return Err("postgres.annotation.initial_template.rejected_initializer");
            }
            _ => return Err("postgres.annotation.initial_template.rejected_control_or_effect"),
        }
    }
    if !file.facts.globals.values().all(independent) {
        return Err("postgres.annotation.initial_template.rejected_global");
    }
    Ok(())
}

fn construct<'a>(
    files: &'a FxHashMap<PathBuf, File<'a>>,
    path: &Path,
    observer: Option<&crate::diagnostics::InvocationObserver>,
) -> Option<Scope> {
    if let Err(reason) = eligibility(&files[path]) {
        count(
            observer,
            "postgres.annotation.initial_template.ineligible_modules",
        );
        count(observer, reason);
        return None;
    }
    // Never recurse through module_environment or resolve imports while the
    // OnceLock is held. The admitted grammar evaluates only inert atoms.
    let mut isolated = super::new(files, |_: &str, _: &Path| None, false);
    let mut env = isolated.environment(fx_map());
    isolated.modules.insert(path.to_path_buf(), env);
    count(observer, "postgres.annotation.module_initializations");
    isolated.steps(&files[path].facts.roots, path, &mut env, 16, true);
    isolated.initialize_module_globals(path, env);
    let scope = isolated.scopes.pop().expect("one independent module frame");
    // Eligibility admits only canonical scalar/function atoms and binding
    // reservations. Those steps cannot allocate a builder, argument object or
    // local frame; module functions therefore all use this registered owner.
    if let Some(observer) = observer {
        observer.increment("postgres.annotation.initial_template.template_builds", 1);
        observer.increment(
            "postgres.annotation.initial_template.cached_bindings",
            scope.len() as u64,
        );
    }
    Some(scope)
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    fn template_context_pristine(&self, env: Environment) -> bool {
        self.scopes[env].is_empty()
            && !self.captured_bindings.contains_key(&env)
            // Other frames pointing TO env have this reverse-reader key. The
            // exact registration/remap index invariant makes this O(1) guard
            // cover captured writes to every inherited reader as well.
            && !self.captured_binding_readers.contains_key(&env)
            // bind_step follows captured ownership above, then synchronizes
            // solely through mapped_arguments[env]. No scan of unrelated
            // argument owners is necessary; freshness must also be absent.
            // With no mapped_arguments[env], sync_mapped_parameter returns
            // (false,false): no escaped callback or fresh reverse-index write
            // can occur. fresh_mapped_argument_bindings is derived metadata,
            // not a separate route for this assignment to reach other frames.
            && !self.mapped_arguments.contains_key(&env)
            && !self.fresh_mapped_parameters.contains_key(&env)
    }

    pub(super) fn apply_initial_template(&mut self, path: &Path, env: Environment) -> bool {
        let observer = self.initial_template_observer.as_deref();
        count(
            observer,
            "postgres.annotation.initial_template.module_activations",
        );
        let files = self.files;
        let Some(template) = files[path]
            .initial_template
            .get_or_init(|| construct(files, path, observer))
        else {
            count(
                observer,
                "postgres.annotation.initial_template.fallback_activations",
            );
            return false;
        };
        count(
            observer,
            "postgres.annotation.initial_template.eligible_activations",
        );
        if !self.template_context_pristine(env) {
            count(
                observer,
                "postgres.annotation.initial_template.context_fallbacks",
            );
            count(
                observer,
                "postgres.annotation.initial_template.fallback_activations",
            );
            return false;
        }
        self.scopes[env] = template.clone();
        count(
            observer,
            "postgres.annotation.initial_template.applied_activations",
        );
        true
    }
}

#[cfg(test)]
mod tests;
