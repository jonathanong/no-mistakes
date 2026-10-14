use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

mod bindings;
pub(super) mod effects;
mod failed_arrays;
mod failed_bindings;
mod failed_values;
mod updates;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn steps(
        &mut self,
        steps: &[Step],
        path: &Path,
        env: &mut Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        self.reserve_bindings(steps, env);
        let supported = !steps.iter().any(|step| {
            matches!(step, Step::Unsupported)
                || (!self.files[path].facts.precise_records
                    && matches!(step, Step::Branch(_, _, _)))
        });
        let unsafe_scope = steps.iter().any(|step| {
            matches!(step, Step::PotentialCalls(calls) if calls.iter().any(|start| !self.files[path].executors.contains(start)))
        });
        let unsupported = if self.files[path].facts.precise_records {
            Value::Failure("unsupported-syntax-or-control-flow")
        } else if unsafe_scope {
            Value::Unknown
        } else {
            Value::Unsupported
        };
        if !supported {
            if self.files[path].facts.precise_records {
                // Unsupported nested flows can mutate builders owned by the caller,
                // even when their local parameter copies are poisoned below.
                self.invalidate_live_builders();
            }
            self.poison_scope(
                *env,
                &unsupported,
                depth,
                self.files[path].facts.precise_records,
            );
        }
        for step in steps {
            if let Step::Hoisted(name, expr) = step {
                let value = self
                    .binding_value(expr, path, env, (depth, generic), (supported, &unsupported))
                    .exposed();
                self.write_captured_binding_evaluated(*env, name, &value, depth);
            }
        }
        let mut opaque_return = self.files[path].facts.precise_records && !supported;
        for (index, step) in steps.iter().enumerate() {
            match step {
                Step::Branch(_, _, _) if !self.files[path].facts.precise_records => {}
                Step::Branch(test, left, right) => {
                    return self.branch_steps(
                        (test, left, right),
                        &steps[index + 1..],
                        path,
                        env,
                        (depth, generic),
                        opaque_return,
                    );
                }
                Step::Bind(name, expr) => {
                    self.bind_step(
                        name,
                        expr,
                        path,
                        env,
                        (depth, generic),
                        (supported, &unsupported),
                    );
                }
                Step::Append(name, expr) => {
                    opaque_return |= self.append_step(
                        name,
                        expr,
                        path,
                        env,
                        (depth, generic),
                        (supported, &unsupported),
                    );
                }
                Step::Effect(expr) => {
                    opaque_return |=
                        self.apply_effect(expr, path, env, depth, generic, &unsupported);
                }
                Step::Return(expr) => {
                    return self.return_step(expr, path, env, (depth, generic), opaque_return);
                }
                Step::Unsupported
                | Step::PotentialCalls(_)
                | Step::Reserve(_)
                | Step::Hoisted(_, _)
                | Step::Var(_) => {}
            }
        }
        if self.files[path].facts.precise_records {
            Value::Evaluated(Box::new(Value::Primitive), !opaque_return)
        } else {
            Value::Unknown
        }
    }

    fn apply_effect(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &mut Environment,
        depth: u8,
        generic: bool,
        unsupported: &Value,
    ) -> bool {
        let effect = self.expr(expr, path, env, depth, generic);
        if self.effect_can_mutate(expr, path, *env)
            && !self.handled_deletion_effect(expr, &effect, path, *env)
            && !self.proven_append_only_effect(expr, path, *env, depth, generic)
        {
            if self.files[path].facts.precise_records
                && (self.modules.get(path).is_some_and(|module| *module == *env)
                    || effects::contains_accessor_read(expr))
            {
                self.invalidate_live_builders();
            }
            // Arbitrary effects can mutate a builder passed by reference.
            // Its previous prefix is no longer proof.
            for value in self.scopes[*env].values_mut() {
                if matches!(value, Value::Prefix(_, _, _)) {
                    *value = if matches!(effect, Value::Unsupported) {
                        unsupported.clone()
                    } else {
                        Value::Unknown
                    };
                }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod effect_tests;
