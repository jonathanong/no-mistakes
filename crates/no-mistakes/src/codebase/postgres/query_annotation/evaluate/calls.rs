pub(super) mod async_suspension;
mod collections;
mod module_loads;
mod names;
pub(super) mod scopes;
mod target;
mod unsupported;
mod values;
use super::statements::effects::contains_accessor_read;
use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(super) fn call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: (u32, u32),
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        self.initialize_require(callee, args, path);
        // Keep the member/index base so a non-arrow callee receives it as `this`.
        // Evaluating the callee expression alone would drop that reference.
        let (target, mut receiver, callee_handled) = self.call_target(callee, path, env, context);
        if self.files[path].facts.precise_records && !callee_handled {
            self.invalidate_live_builders();
        }
        let mut raw_arguments = args
            .iter()
            .map(|arg| self.operand(arg, path, env, context))
            .collect::<Vec<_>>();
        // Later arguments run before the executor. A getter or opaque call can
        // mutate the SQL builder already evaluated as the first argument.
        if args
            .iter()
            .skip(1)
            .any(|arg| contains_accessor_read(arg) && self.effect_can_mutate(arg, path, *env))
        {
            self.invalidate_live_builders();
        }
        if let Some(receiver) = receiver.as_mut() {
            // A method was selected before its arguments ran, but `this` may
            // still hold a builder changed by an argument expression.
            self.refresh_arguments(std::slice::from_mut(receiver));
        }
        // Later operands can mutate the object an earlier operand returned.
        self.refresh_arguments(&mut raw_arguments);
        if self.files[path].facts.precise_records
            && args.iter().zip(&raw_arguments).any(|(arg, value)| {
                matches!(
                    arg,
                    Expr::Index(_, _) | Expr::Member(_, _) | Expr::ComputedMember(_, _)
                ) && matches!(value, Value::Prefix(_, _, Some(_)))
            })
        {
            // Preserve the existing indirect-builder argument boundary. Array
            // evaluation does not authorize accessor-based builder forwarding.
            self.invalidate_builders(&raw_arguments);
        }
        let handled_projection = matches!(callee, Expr::Name(_))
            && self.projected_primitive_arguments(args, &raw_arguments, path, *env);
        let executor_operands_opaque = !callee_handled
            || args
                .iter()
                .zip(&raw_arguments)
                .enumerate()
                .any(|(index, (arg, value))| {
                    matches!(value, Value::Evaluated(_, false))
                        || (index > 0 && !self.handled_deletion_effect(arg, value, path, *env))
                });
        let operands_handled = callee_handled
            && args
                .iter()
                .zip(&raw_arguments)
                .all(|(arg, value)| self.handled_deletion_effect(arg, value, path, *env));
        let arguments = raw_arguments
            .into_iter()
            .map(Value::exposed)
            .collect::<Vec<_>>();
        if self.files[path].executors.contains(&span) {
            let value = if self.files[path].facts.precise_records && executor_operands_opaque {
                self.invalidate_live_builders();
                Value::Failure("opaque-executor-operand-effect")
            } else {
                arguments.first().cloned().unwrap_or(Value::Unknown)
            };
            self.events
                .entry((path.to_path_buf(), span))
                .or_default()
                .push((generic, value));
            return Value::Unknown;
        }
        if let Some(value) = self.collection_projection(
            callee,
            args,
            receiver.as_ref(),
            &arguments,
            path,
            (depth, generic, operands_handled),
        ) {
            return value;
        }
        if self.files[path].facts.precise_records && !operands_handled {
            self.invalidate_live_builders();
            self.opaque_callbacks(&arguments, depth);
            return Value::Failure("opaque-call-operand-effect");
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
            self.untrusted_array_intrinsics |= !handled_projection;
            self.invalidate_builders(&arguments);
            self.opaque_callbacks(&arguments, depth);
            return if handled_projection {
                Value::Evaluated(Box::new(Value::Unknown), true)
            } else {
                Value::Unknown
            };
        };
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
                &arguments,
                receiver.as_ref(),
                (captured, depth, spread),
            );
        }
        let mut locals = scopes::locals(&self.scopes[captured], &function);
        if let Some(name) = &function.self_name {
            locals.insert(name.clone(), Value::Unknown);
        }
        let object = self.arguments_object(&arguments, spread);
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
        scopes::bind_this(&mut locals, &function, receiver.as_ref());
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
            self.invalidate_builders(&arguments);
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
