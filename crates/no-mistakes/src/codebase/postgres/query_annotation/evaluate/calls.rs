use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation) fn name(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
    ) -> Value {
        let file = &self.files[path];
        if let Some(expr) = file.facts.globals.get(name).cloned() {
            let env = self.environment(Default::default());
            return self.expr(&expr, path, &env, depth, generic);
        }
        let import = file
            .ts
            .imported_bindings
            .iter()
            .find(|binding| binding.local == name && !binding.is_type_only)
            .cloned();
        let Some(import) = import else {
            return Value::Unknown;
        };
        let Some(target) = (self.resolve)(&import.specifier, path) else {
            return Value::Unknown;
        };
        self.export(&target, &import.imported, depth, generic)
    }

    fn export(&mut self, path: &Path, name: &str, depth: u8, generic: bool) -> Value {
        self.lookup_export(path, name, depth, generic)
            .unwrap_or(Value::Unknown)
    }

    pub(super) fn call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        start: u32,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let arguments = args
            .iter()
            .map(|arg| self.expr(arg, path, env, depth, generic))
            .collect::<Vec<_>>();
        if self.files[path].executors.contains(&start) {
            self.events
                .entry((path.to_path_buf(), start))
                .or_default()
                .push((
                    generic,
                    arguments.first().cloned().unwrap_or(Value::Unknown),
                ));
            return Value::Unknown;
        }
        let target = self.expr(callee, path, env, depth, generic);
        if matches!(target, Value::Unsupported) {
            self.opaque_callbacks(&arguments, depth);
            return Value::Unsupported;
        }
        let Value::Function(function, function_path, captured) = target else {
            self.opaque_callbacks(&arguments, depth);
            return Value::Unknown;
        };
        if !function.supported {
            let mut locals = self.scopes[captured].clone();
            for name in &function.params {
                locals.insert(name.clone(), Value::Unknown);
            }
            if let Some(name) = &function.self_name {
                locals.insert(name.clone(), Value::Unknown);
            }
            let mut scope = self.environment(locals);
            self.steps(&function.body, &function_path, &mut scope, depth, false);
            self.opaque_callbacks(&arguments, depth);
            return Value::Unsupported;
        }
        let mut locals = self.scopes[captured].clone();
        for (index, param) in function.params.iter().enumerate() {
            locals.insert(
                param.clone(),
                arguments.get(index).cloned().unwrap_or(Value::Unknown),
            );
        }
        if let Some(name) = &function.self_name {
            locals.insert(name.clone(), Value::Unknown);
        }
        let mut scope = self.environment(locals);
        let value = self.steps(&function.body, &function_path, &mut scope, depth, false);
        // Trace forwarded executor calls, but do not treat a helper's return
        // as immutable SQL when arbitrary effects could mutate its builder.
        if !function.asynchronous {
            value
        } else {
            Value::Unknown
        }
    }

    fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        for argument in arguments {
            if let Value::Function(function, path, captured) = argument {
                let mut locals = self.scopes[*captured].clone();
                for name in &function.params {
                    locals.insert(name.clone(), Value::Unknown);
                }
                if let Some(name) = &function.self_name {
                    locals.insert(name.clone(), Value::Unknown);
                }
                let mut scope = self.environment(locals);
                self.steps(&function.body, path, &mut scope, depth, false);
            }
        }
    }
}
