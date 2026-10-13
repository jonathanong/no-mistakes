use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn steps(
        &mut self,
        steps: &[Step],
        path: &Path,
        env: &mut Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
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
                    let value = if supported {
                        self.expr(expr, path, env, depth, generic).exposed()
                    } else {
                        unsupported.clone()
                    };
                    self.write_captured_binding_evaluated(*env, name, &value, depth);
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
                    let effect = self.expr(expr, path, env, depth, generic);
                    if self.effect_can_mutate(expr, path, *env)
                        && !self.handled_deletion_effect(expr, &effect, path, *env)
                        && !self.proven_append_only_effect(expr, path, depth, generic)
                    {
                        opaque_return = true;
                        // Arbitrary effects can mutate a builder passed by
                        // reference. Its previous prefix is no longer proof.
                        for value in self.scopes[*env].values_mut() {
                            if matches!(value, Value::Prefix(_, _, _)) {
                                *value = if matches!(effect, Value::Unsupported) {
                                    unsupported.clone()
                                } else {
                                    Value::Unknown
                                };
                            }
                        }
                    }
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

    fn proven_append_only_effect(
        &mut self,
        expr: &Expr,
        path: &Path,
        depth: u8,
        generic: bool,
    ) -> bool {
        let Expr::Call { callee, .. } = expr else {
            return false;
        };
        let Expr::Name(name) = &**callee else {
            return false;
        };
        let Value::Function(function, function_path, captured) =
            self.name(path, name, depth, generic)
        else {
            return false;
        };
        function.supported && function.body.iter().all(|step| match step {
            Step::Reserve(_) => true,
            Step::Append(target, tail) => {
                function.params.contains(target)
                    && match tail {
                        Expr::Text(_) => true,
                        Expr::Template(parts) => {
                            parts.iter().all(|part| matches!(part, Expr::Text(_)))
                        }
                        Expr::Tagged(tag, parts, effects) => {
                            self.tag_trusted(tag, &function_path, captured)
                                && parts.iter().all(|part| matches!(part, Expr::Text(_)))
                                && matches!(effects.as_slice(), [Expr::Name(name)] if name == tag)
                        }
                        _ => false,
                    }
            }
            _ => false,
        })
    }

    pub(super) fn effect_can_mutate(&self, expr: &Expr, path: &Path, env: Environment) -> bool {
        match expr {
            Expr::Call { start, end, .. } => !self.files[path].executors.contains(&(*start, *end)),
            Expr::Template(parts)
            | Expr::Children(parts)
            | Expr::Container(parts)
            | Expr::Object(parts)
            | Expr::Record(parts, _)
            | Expr::Sequence(parts) => parts
                .iter()
                .any(|part| self.effect_can_mutate(part, path, env)),
            Expr::Tagged(tag, _, _) => !self.tag_trusted(tag, path, env),
            Expr::Await(expr) => self.effect_can_mutate(expr, path, env),
            // A getter or proxy trap can mutate captured state. Numeric argument
            // slots and `length` are data. Any other property can be an accessor.
            Expr::Index(expr, _) => {
                self.effect_can_mutate(expr, path, env)
                    || !self.proven_argument_data_read(expr, env)
            }
            Expr::Member(expr, name) => {
                self.effect_can_mutate(expr, path, env)
                    || !(name == "length" && self.proven_argument_data_read(expr, env))
            }
            Expr::Discard(expr) => self.effect_can_mutate(expr, path, env),
            Expr::Primitive | Expr::Text(_) | Expr::Name(_) | Expr::Function(_) => false,
            _ => true,
        }
    }

    /// True when `receiver` is the invocation's argument object, including an
    /// alias such as `const slots = arguments`. A parameter that shadows the
    /// name `arguments` is an ordinary value and is not included.
    fn proven_argument_data_read(&self, receiver: &Expr, env: Environment) -> bool {
        let Expr::Name(name) = receiver else {
            return false;
        };
        self.scopes
            .get(env)
            .and_then(|scope| scope.get(name))
            .is_some_and(is_arguments_value)
    }
}

fn is_arguments_value(value: &Value) -> bool {
    match value {
        Value::Arguments(_) => true,
        Value::Evaluated(inner, _) => is_arguments_value(inner),
        _ => false,
    }
}

#[cfg(test)]
mod effect_tests;
