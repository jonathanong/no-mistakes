use super::*;
use crate::codebase::postgres::query_annotation::Function;
use std::{path::PathBuf, sync::Arc};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invoke_known_function(
        &mut self,
        function: Arc<Function>,
        function_path: PathBuf,
        captured: Environment,
        arguments: &[Value],
        receiver: Option<&Value>,
        context: (u8, bool),
    ) -> Value {
        let (depth, spread) = context;
        if !function.supported
            || (!self.files[&function_path].facts.precise_records
                && function.body.iter().any(|step| {
                    matches!(
                        step,
                        crate::codebase::postgres::query_annotation::Step::Branch(_, _, _)
                    )
                }))
        {
            return self.unsupported_call(
                &function,
                &function_path,
                arguments,
                receiver,
                (captured, depth, spread),
            );
        }
        let mut locals = scopes::locals(&self.scopes[captured], &function);
        if let Some(name) = &function.self_name {
            locals.insert(name.clone(), Value::Unknown);
        }
        let object = self.arguments_object(arguments, spread);
        let mapping = self.parameter_mapping(&function, &function_path, &object);
        scopes::arguments(&mut locals, &function, object);
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
        scopes::bind_this(&mut locals, &function, receiver);
        let mut scope = self.environment(locals);
        self.register_mappings(scope, captured, &function, mapping);
        self.register_captured_bindings(scope, captured, &function);
        let suspended = self.files[&function_path].facts.precise_records
            && function.asynchronous
            && async_suspension::has_suspension(&function.body);
        let prior_events = suspended.then(|| async_suspension::event_lengths(&self.events));
        let value = self.steps(&function.body, &function_path, &mut scope, depth, false);
        if let Some(prior_events) = prior_events {
            // The evaluator does not schedule post-await work. Whether the
            // caller awaits this promise or reads a shared builder first is
            // unknown. Events recorded while evaluating the body are just as
            // uncertain as its return value and captured builders.
            async_suspension::invalidate_new_events(&mut self.events, &prior_events);
            self.invalidate_builders(arguments);
            self.invalidate_captured(captured, &function);
            return Value::Promise(Box::new(Value::Unknown));
        }
        // Trace forwarded executor calls, but do not treat a helper's return
        // as immutable SQL when arbitrary effects could mutate its builder.
        if !function.asynchronous {
            value
        } else {
            Value::Promise(Box::new(value))
        }
    }
}
