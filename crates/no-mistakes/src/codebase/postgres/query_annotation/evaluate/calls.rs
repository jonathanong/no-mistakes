use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(super) fn name(&mut self, path: &Path, name: &str, depth: u8, generic: bool) -> Value {
        let file = &self.files[path];
        if let Some(expr) = file.facts.globals.get(name).cloned() {
            return self.expr(&expr, path, &Environment::new(), depth, generic);
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
        let Some(depth) = depth.checked_sub(1) else {
            return Value::Unknown;
        };
        let Some(file) = self.files.get(path) else {
            return Value::Unknown;
        };
        let binding = file
            .ts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == name)
            .cloned();
        let Some(binding) = binding else {
            return Value::Unknown;
        };
        if let Some(specifier) = binding.specifier {
            let Some(target) = (self.resolve)(&specifier, path) else {
                return Value::Unknown;
            };
            self.export(&target, &binding.local, depth, generic)
        } else {
            self.name(path, &binding.local, depth, generic)
        }
    }

    pub(super) fn call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        site: (u32, &str),
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (line, spelling) = site;
        let (depth, generic) = context;
        let arguments = args
            .iter()
            .map(|arg| self.expr(arg, path, env, depth, generic))
            .collect::<Vec<_>>();
        if self.files[path]
            .executors
            .iter()
            .any(|(site_line, name)| *site_line == line && name == spelling)
        {
            self.events
                .entry((path.to_path_buf(), line, spelling.to_string()))
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
        let Value::Function(function, function_path, mut locals) = target else {
            self.opaque_callbacks(&arguments, depth);
            return Value::Unknown;
        };
        if !function.supported {
            self.opaque_callbacks(&arguments, depth);
            return Value::Unsupported;
        }
        for (index, param) in function.params.iter().enumerate() {
            locals.insert(
                param.clone(),
                arguments.get(index).cloned().unwrap_or(Value::Unknown),
            );
        }
        if let Some(name) = &function.self_name {
            locals.insert(name.clone(), Value::Unknown);
        }
        let value = self.steps(&function.body, &function_path, &mut locals, depth, false);
        // Trace forwarded executor calls, but do not treat a helper's return
        // as immutable SQL when arbitrary effects could mutate its builder.
        if !function.asynchronous
            && !function.body.iter().any(|step| {
                matches!(
                    step,
                    crate::codebase::postgres::query_annotation::Step::Effect(_)
                )
            })
        {
            value
        } else {
            Value::Unknown
        }
    }

    fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        for argument in arguments {
            if let Value::Function(function, path, captured) = argument {
                if !function.supported {
                    continue;
                }
                let mut locals = captured.clone();
                for name in &function.params {
                    locals.insert(name.clone(), Value::Unknown);
                }
                if let Some(name) = &function.self_name {
                    locals.insert(name.clone(), Value::Unknown);
                }
                self.steps(&function.body, path, &mut locals, depth, false);
            }
        }
    }
}
