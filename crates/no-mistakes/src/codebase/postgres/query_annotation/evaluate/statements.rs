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
        // Reserve lexical names before evaluation so TDZ/shadowed bindings do
        // not accidentally resolve to a module helper or trusted tag.
        for step in steps {
            match step {
                Step::Bind(name, _) | Step::Hoisted(name, _) => {
                    self.scopes[*env].insert(name.clone(), Value::Unknown);
                }
                Step::Reserve(names) => {
                    for name in names {
                        self.scopes[*env].insert(name.clone(), Value::Unknown);
                    }
                }
                _ => {}
            }
        }
        let supported = !steps.iter().any(|step| matches!(step, Step::Unsupported));
        if !supported {
            for value in self.scopes[*env].values_mut() {
                *value = Value::Unsupported;
            }
        }
        for step in steps {
            if let Step::Hoisted(name, expr) = step {
                let value = if supported {
                    self.expr(expr, path, env, depth, generic)
                } else {
                    Value::Unsupported
                };
                self.scopes[*env].insert(name.clone(), value);
            }
        }
        let mut opaque_return = false;
        for step in steps {
            match step {
                Step::Bind(name, expr) => {
                    let value = if supported {
                        self.expr(expr, path, env, depth, generic)
                    } else {
                        Value::Unsupported
                    };
                    self.scopes[*env].insert(name.clone(), value);
                }
                Step::Append(name, expr) => {
                    let tail = self.expr(expr, path, env, depth, generic);
                    let base = self.scopes[*env]
                        .get(name)
                        .cloned()
                        .unwrap_or(Value::Unknown);
                    self.scopes[*env].insert(
                        name.clone(),
                        if supported {
                            concat(base, tail)
                        } else {
                            Value::Unsupported
                        },
                    );
                }
                Step::Effect(expr) => {
                    let effect = self.expr(expr, path, env, depth, generic);
                    if self.effect_can_mutate(expr, path) {
                        opaque_return = true;
                        // Arbitrary effects can mutate a builder passed by
                        // reference. Its previous prefix is no longer proof.
                        for value in self.scopes[*env].values_mut() {
                            if matches!(value, Value::Prefix(_, _)) {
                                *value = if matches!(effect, Value::Unsupported) {
                                    Value::Unsupported
                                } else {
                                    Value::Unknown
                                };
                            }
                        }
                    }
                }
                Step::Return(expr) => {
                    let value = self.expr(expr, path, env, depth, generic);
                    return if opaque_return { Value::Unknown } else { value };
                }
                Step::Unsupported | Step::Reserve(_) | Step::Hoisted(_, _) => {}
            }
        }
        Value::Unknown
    }

    fn effect_can_mutate(&self, expr: &Expr, path: &Path) -> bool {
        match expr {
            Expr::Call { start, .. } => !self.files[path].executors.contains(start),
            Expr::Template(parts) | Expr::Children(parts) => {
                parts.iter().any(|part| self.effect_can_mutate(part, path))
            }
            Expr::Text(_) | Expr::Name(_) | Expr::Function(_) | Expr::Tagged(_, _) => false,
            _ => true,
        }
    }
}
