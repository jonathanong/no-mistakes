use super::{scopes, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Function;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn unsupported_call(
        &mut self,
        function: &Function,
        function_path: &Path,
        arguments: &[Value],
        receiver: Option<&Value>,
        context: (Environment, u8, bool),
    ) -> Value {
        let (captured, depth, spread) = context;
        self.invalidate_builders(arguments);
        self.invalidate_captured(captured, function);
        let mut locals = scopes::locals(&self.scopes[captured], function);
        if let Some(name) = &function.self_name {
            locals.insert(name.clone(), Value::Unknown);
        }
        let object = self.arguments_object(arguments, spread);
        let mapping = self.parameter_mapping(function, function_path, &object);
        scopes::arguments(&mut locals, function, object);
        for name in &function.params {
            locals.insert(name.clone(), Value::Unknown);
        }
        scopes::bind_this(&mut locals, function, receiver);
        let mut scope = self.environment(locals);
        self.register_mappings(scope, captured, function, mapping);
        self.register_captured_bindings(scope, captured, function);
        self.steps(&function.body, function_path, &mut scope, depth, false);
        self.opaque_callbacks(arguments, depth);
        // Unsupported control flow with possible opaque calls must not
        // revive a legacy SQL prefix that its effects could have invalidated.
        let unsafe_calls = function.body.iter().any(|step| {
            use crate::codebase::postgres::query_annotation::Step;
            match step {
                Step::PotentialCalls(calls) => calls
                    .iter()
                    .any(|start| !self.files[function_path].executors.contains(start)),
                Step::Effect(expr) | Step::Bind(_, expr) | Step::Return(expr) => {
                    self.effect_can_mutate(expr, function_path, scope)
                }
                Step::Append(_, _) => true,
                _ => false,
            }
        });
        if unsafe_calls {
            Value::Unknown
        } else {
            Value::Unsupported
        }
    }
}
