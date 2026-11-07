use super::super::{Environment, Evaluator, File, Scope, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

pub(in crate::codebase::postgres::query_annotation) struct SharedInitials {
    values: DashMap<PathBuf, Arc<OnceLock<Option<Frozen>>>, FxBuildHasher>,
}

#[derive(Eq, PartialEq)]
struct Frozen {
    scope: Scope,
    environment: Environment,
    invalidates: bool,
    untrusted: bool,
}

impl SharedInitials {
    pub(in crate::codebase::postgres::query_annotation) fn new() -> Arc<Self> {
        Arc::new(Self {
            values: DashMap::with_hasher(FxBuildHasher),
        })
    }
}

fn eligible(file: &File<'_>) -> bool {
    file.executors.is_empty()
        && file.module.imports.is_empty()
        && file.module.ts.imports.is_empty()
        && file.module.ts.eager_require_specifiers.is_empty()
        && file.module.ts.star_reexport_specifiers.is_empty()
        && file
            .module
            .ts
            .exported_bindings
            .iter()
            .all(|binding| binding.specifier.is_none())
        && file
            .facts
            .roots
            .iter()
            .all(|step| scalar_step(step, Some(&file.facts.globals)))
}

fn scalar_step(step: &Step, globals: Option<&BTreeMap<String, Expr>>) -> bool {
    match step {
        Step::Reserve(_) | Step::Var(_) => true,
        Step::Hoisted(_, Expr::Function(function)) | Step::Bind(_, Expr::Function(function)) => {
            globals.is_some()
                && function.supported
                && !function.asynchronous
                && function.body.iter().all(|step| scalar_step(step, None))
        }
        Step::Bind(_, expr) | Step::Effect(expr) | Step::Return(expr) => scalar_expr(expr, globals),
        _ => false,
    }
}

fn scalar_expr(expr: &Expr, globals: Option<&BTreeMap<String, Expr>>) -> bool {
    match expr {
        Expr::Text(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Primitive | Expr::Unknown => {
            true
        }
        Expr::Name(name) => name != "arguments",
        Expr::Template(parts) | Expr::Sequence(parts) => {
            parts.iter().all(|part| scalar_expr(part, globals))
        }
        Expr::Binary(operator, left, right) => {
            operator == "+" && scalar_expr(left, globals) && scalar_expr(right, globals)
        }
        Expr::Discard(expr) => scalar_expr(expr, globals),
        Expr::Call { callee, args, .. } => {
            let Some(globals) = globals else { return false };
            let Expr::Name(name) = &**callee else {
                return false;
            };
            (args.is_empty() || matches!(globals.get(name), Some(Expr::Function(_))))
                && args.iter().all(|arg| scalar_expr(arg, Some(globals)))
        }
        _ => false,
    }
}

fn portable(value: &Value, environment: Environment) -> bool {
    match value {
        Value::Prefix(_, _, None)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::Failure(_)
        | Value::Unknown
        | Value::Primitive
        | Value::Unsupported => true,
        Value::Function(_, _, captured) => *captured == environment,
        _ => false,
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn shared_module_initial(
        &mut self,
        path: &Path,
    ) -> Option<Environment> {
        let shared = Arc::clone(self.modules.shared_initials.as_ref()?);
        if self.files[path].facts.root_eligible != Some(false) {
            return None;
        }
        let entry = match shared.values.get(path) {
            Some(entry) => Arc::clone(entry.value()),
            None => Arc::clone(shared.values.entry(path.to_path_buf()).or_default().value()),
        };
        let frozen = entry
            .get_or_init(|| {
                if !eligible(&self.files[path]) {
                    return None;
                }
                // Eligibility forbids module loads; a fixed type also bounds monomorphization.
                let resolve: fn(&str, &Path) -> Option<PathBuf> = |_, _| None;
                let mut evaluator =
                    super::super::super::project::evaluator::new(self.files, resolve, false);
                let environment = evaluator.module_environment(path);
                let scope = &evaluator.scopes[environment];
                if !scope.values().all(|value| portable(value, environment)) {
                    return None;
                }
                if let Some(observer) = crate::diagnostics::current() {
                    observer.increment("postgres.annotation.shared_module_initializations", 1);
                }
                Some(Frozen {
                    scope: scope.clone(),
                    environment,
                    invalidates: evaluator.modules.invalidates_external_builders,
                    untrusted: evaluator.untrusted_array_intrinsics,
                })
            })
            .as_ref()?;
        let environment = self.scopes.len();
        let mut scope = frozen.scope.clone();
        if environment != frozen.environment {
            for value in scope.values_mut() {
                if let Value::Function(_, _, captured) = value {
                    *captured = environment;
                }
            }
        }
        self.scopes.push(scope);
        let module_id = self.module_ids().id(path);
        self.modules.insert(module_id, environment);
        self.untrusted_array_intrinsics |= frozen.untrusted;
        if frozen.invalidates {
            self.invalidate_live_builders();
        }
        self.record_alternative_module_initial(path, environment);
        Some(environment)
    }
}
