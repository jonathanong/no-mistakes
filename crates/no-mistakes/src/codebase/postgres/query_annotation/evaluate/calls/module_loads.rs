use super::{Environment, Evaluator, Expr, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn await_expr(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        if let Expr::ModuleLoad(specifier) = expr {
            return self.await_dynamic_import(specifier, path);
        }
        let mut value = self.expr(expr, path, env, depth, generic).exposed();
        while let Value::Promise(inner) = value {
            value = inner.exposed();
        }
        value
    }

    pub(super) fn initialize_require(&mut self, callee: &Expr, args: &[Expr], path: &Path) {
        if !matches!(callee, Expr::Name(name) if name == "require")
            || !self.files[path].facts.precise_records
        {
            return;
        }
        let target = match args {
            [Expr::Text(specifier)]
                if self.files[path]
                    .module
                    .ts
                    .eager_require_specifiers
                    .contains(specifier) =>
            {
                self.files[path]
                    .module
                    .target(specifier, path, self.resolver())
                    .map(Path::to_path_buf)
            }
            _ => None,
        };
        self.initialize_runtime_module(target);
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn load_dynamic_import(
        &mut self,
        _specifier: &str,
        path: &Path,
    ) -> Value {
        if !self.files[path].facts.precise_records {
            return Value::Unknown;
        }
        // `import()` schedules module evaluation after the current job. Even
        // a literal supplied target must not be executed synchronously here:
        // reads before it runs and reads after an await can see different SQL.
        self.invalidate_live_builders();
        Value::Promise(Box::new(Value::Unknown))
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn await_dynamic_import(
        &mut self,
        specifier: &str,
        path: &Path,
    ) -> Value {
        if !self.files[path].facts.precise_records {
            return Value::Unknown;
        }
        let target = self.files[path]
            .module
            .target(specifier, path, self.resolver())
            .map(Path::to_path_buf);
        self.initialize_runtime_module(target);
        Value::Unknown
    }

    fn initialize_runtime_module(&mut self, target: Option<PathBuf>) {
        if let Some(target) = target {
            self.module_environment(&target);
        } else {
            // An unresolved runtime module can mutate builders owned by any
            // previously initialized supplied module.
            self.invalidate_live_builders();
        }
    }
}
