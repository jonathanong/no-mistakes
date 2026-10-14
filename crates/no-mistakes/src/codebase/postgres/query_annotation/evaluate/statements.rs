use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

mod effects;

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
        let supported = !steps.iter().any(|step| matches!(step, Step::Unsupported));
        let unsafe_scope = steps.iter().any(|step| {
            matches!(step, Step::PotentialCalls(calls) if calls.iter().any(|start| !self.files[path].executors.contains(start)))
        });
        let unsupported = if unsafe_scope {
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
        for step in steps {
            match step {
                Step::Bind(name, expr) => {
                    let may_mutate = self.files[path].facts.precise_records
                        && effects::contains_accessor_read(expr)
                        && self.effect_can_mutate(expr, path, *env)
                        && !self.proven_append_only_effect(expr, path, *env, depth, generic);
                    let live_before = may_mutate.then(|| self.live_builder_values());
                    let value = if supported {
                        self.expr(expr, path, env, depth, generic).exposed()
                    } else {
                        unsupported.clone()
                    };
                    self.write_captured_binding_evaluated(*env, name, &value, depth);
                    if let Some(live_before) = live_before {
                        self.invalidate_builders(&live_before);
                    }
                }
                Step::Append(name, expr) => {
                    let tail = self.expr(expr, path, env, depth, generic).exposed();
                    let base = self.scopes[*env]
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Unknown);
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
                    let value = self.expr(expr, path, env, depth, generic).exposed();
                    return if opaque_return { Value::Unknown } else { value };
                }
                Step::Unsupported
                | Step::PotentialCalls(_)
                | Step::Reserve(_)
                | Step::Hoisted(_, _)
                | Step::Var(_) => {}
            }
        }
        Value::Unknown
    }

    fn reserve_bindings(&mut self, steps: &[Step], env: &Environment) {
        let vars = steps
            .iter()
            .filter_map(|step| match step {
                Step::Var(name) => Some(name.as_str()),
                _ => None,
            })
            .collect::<crate::fx::FxHashSet<_>>();
        // Reserve lexical names before evaluation so TDZ/shadowed bindings do
        // not accidentally resolve to a module helper or trusted tag.
        for step in steps {
            match step {
                Step::Bind(name, _) if vars.contains(name.as_str()) => {
                    // Initialized var redeclarations assign at this statement,
                    // rather than replacing a parameter during hoisting.
                    self.scopes[*env]
                        .entry(name.clone())
                        .or_insert(Value::Unknown);
                }
                Step::Bind(name, _) | Step::Hoisted(name, _) => {
                    self.scopes[*env].insert(name.clone(), Value::Unknown);
                }
                Step::Var(name) => {
                    self.scopes[*env]
                        .entry(name.clone())
                        .or_insert(Value::Unsupported);
                }
                Step::Reserve(names) => {
                    for name in names {
                        self.scopes[*env]
                            .entry(name.clone())
                            .or_insert(Value::Unknown);
                    }
                }
                _ => {}
            }
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
