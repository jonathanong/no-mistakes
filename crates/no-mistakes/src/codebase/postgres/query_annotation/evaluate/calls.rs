mod scopes;
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
        if file.facts.globals.contains_key(name) {
            if matches!(file.facts.globals.get(name), Some(Expr::Unknown)) {
                return Value::Unknown;
            }
            let env = self.module_environment(path);
            return self.scopes[env]
                .get(name)
                .cloned()
                .unwrap_or(Value::Unknown);
        }
        let import = file
            .imports
            .get(name)
            .map(|index| file.ts.imported_bindings[*index].clone());
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
        let target = self.expr(callee, path, env, depth, generic);
        let mut arguments = args
            .iter()
            .map(|arg| self.expr(arg, path, env, depth, generic))
            .collect::<Vec<_>>();
        for argument in &mut arguments {
            if matches!(argument, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
            {
                *argument = Value::Unknown;
            }
        }
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
        let spread = args.iter().any(|arg| matches!(arg, Expr::Spread(_)));
        if spread {
            // Values may mutate builders, but spread length cannot prove any
            // positional parameter or callback substitution.
            self.invalidate_builders(&arguments);
            self.opaque_callbacks(&arguments, depth);
        }
        if !matches!(target, Value::Function(_, _, _)) {
            self.invalidate_builders(std::slice::from_ref(&target));
            self.opaque_callbacks(std::slice::from_ref(&target), depth);
        }
        if matches!(target, Value::Unsupported) {
            self.invalidate_builders(&arguments);
            self.opaque_callbacks(&arguments, depth);
            return Value::Unsupported;
        }
        let Value::Function(function, function_path, captured) = target else {
            self.invalidate_builders(&arguments);
            self.opaque_callbacks(&arguments, depth);
            return Value::Unknown;
        };
        if !function.supported {
            self.invalidate_builders(&arguments);
            self.invalidate_captured(captured, &function);
            let mut locals = scopes::locals(&self.scopes[captured], &function);
            scopes::arguments(&mut locals, &function, Value::Arguments(arguments.clone()));
            for name in &function.params {
                locals.insert(name.clone(), Value::Unknown);
            }
            if let Some(name) = &function.self_name {
                locals.insert(name.clone(), Value::Unknown);
            }
            let mut scope = self.environment(locals);
            self.steps(&function.body, &function_path, &mut scope, depth, false);
            self.opaque_callbacks(&arguments, depth);
            // Unsupported control flow with possible opaque calls must not
            // revive a legacy SQL prefix that its effects could have invalidated.
            let unsafe_calls = function.body.iter().any(|step| {
                use crate::codebase::postgres::query_annotation::Step;
                match step {
                    Step::PotentialCalls(calls) => calls
                        .iter()
                        .any(|start| !self.files[&function_path].executors.contains(start)),
                    Step::Effect(expr) | Step::Bind(_, expr) | Step::Return(expr) => {
                        self.effect_can_mutate(expr, &function_path, scope)
                    }
                    Step::Append(_, _) => true,
                    _ => false,
                }
            });
            return if unsafe_calls {
                Value::Unknown
            } else {
                Value::Unsupported
            };
        }
        let mut locals = scopes::locals(&self.scopes[captured], &function);
        scopes::arguments(&mut locals, &function, Value::Arguments(arguments.clone()));
        for (index, param) in function.params.iter().enumerate() {
            locals.insert(
                param.clone(),
                if spread {
                    Value::Unknown
                } else {
                    arguments.get(index).cloned().unwrap_or(Value::Unknown)
                },
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
            Value::Promise(Box::new(value))
        }
    }

    pub(super) fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        for argument in arguments {
            if let Value::Aggregate(values) | Value::Arguments(values) = argument {
                self.opaque_callbacks(values, depth);
            } else if let Value::Promise(value) = argument {
                self.opaque_callbacks(std::slice::from_ref(value.as_ref()), depth);
            } else if let Value::Function(function, path, captured) = argument {
                self.invalidate_captured(*captured, function);
                let mut locals = scopes::locals(&self.scopes[*captured], function);
                scopes::arguments(&mut locals, function, Value::Unknown);
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
