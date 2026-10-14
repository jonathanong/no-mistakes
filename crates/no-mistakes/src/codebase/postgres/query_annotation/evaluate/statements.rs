use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

mod bindings;
pub(super) mod effects;

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
            for value in self.scopes[*env].values_mut() {
                *value = unsupported.clone();
            }
        }
        for step in steps {
            if let Step::Hoisted(name, expr) = step {
                let value = if supported {
                    self.expr(expr, path, env, depth, generic).exposed()
                } else {
                    unsupported.clone()
                };
                self.write_captured_binding_evaluated(*env, name, &value, depth);
            }
        }
        let mut opaque_return = false;
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
                    let may_mutate = self.files[path].facts.precise_records
                        && (effects::contains_accessor_read(expr)
                            || matches!(expr, Expr::Call { .. }))
                        && self.effect_can_mutate(expr, path, *env)
                        && !self.proven_append_only_effect(expr, path, *env, depth, generic);
                    let live_before = may_mutate.then(|| self.live_builder_values());
                    let value = if supported {
                        self.expr(expr, path, env, depth, generic)
                    } else {
                        unsupported.clone()
                    };
                    let handled = self.handled_deletion_effect(expr, &value, path, *env);
                    let value = value.exposed();
                    self.write_captured_binding_evaluated(*env, name, &value, depth);
                    if let Some(live_before) = live_before {
                        // Known function bodies have already projected their
                        // builder effects. An opaque result or accessor read
                        // still cannot prove that earlier builders survived.
                        if !handled
                            && (effects::contains_accessor_read(expr)
                                || matches!(
                                    value,
                                    Value::Unknown | Value::Unsupported | Value::Failure(_)
                                ))
                        {
                            self.invalidate_builders(&live_before);
                        }
                    }
                }
                Step::Append(name, expr) => {
                    let tail = self.expr(expr, path, env, depth, generic).exposed();
                    let base = self.scopes[*env]
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Unknown);
                    if self.files[path].facts.precise_records
                        && !matches!(&base, Value::Prefix(_, _, Some(id)) if !self.invalidated_builders.contains(id))
                        && !matches!(&base, Value::Joined(values) if values.iter().all(|value| matches!(value, Value::Prefix(_, _, Some(id)) if !self.invalidated_builders.contains(id))))
                    {
                        opaque_return = true;
                        self.invalidate_live_builders();
                        self.opaque_callbacks(std::slice::from_ref(&base), depth);
                    }
                    let value = if supported {
                        concat(base, tail)
                    } else {
                        unsupported.clone()
                    };
                    self.replace_builder(&value);
                    self.write_captured_binding(*env, name, &value);
                }
                Step::Effect(expr) => {
                    opaque_return |=
                        self.apply_effect(expr, path, env, depth, generic, &unsupported);
                }
                Step::Return(expr) => {
                    let result = self.expr(expr, path, env, depth, generic);
                    if self.files[path].facts.precise_records {
                        if matches!(result.clone().exposed(), Value::Failure(_)) {
                            return result.exposed();
                        }
                        let handled = self.handled_deletion_effect(expr, &result, path, *env);
                        if matches!(&result, Value::Evaluated(_, false))
                            || opaque_return
                            || (self.effect_can_mutate(expr, path, *env) && !handled)
                        {
                            return Value::Failure("opaque-return-effect");
                        }
                        return Value::Evaluated(Box::new(result.exposed()), true);
                    }
                    return if opaque_return {
                        Value::Unknown
                    } else {
                        result.exposed()
                    };
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
