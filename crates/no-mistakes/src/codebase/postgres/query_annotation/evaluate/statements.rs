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
            if let Step::Bind(name, _) = step {
                env.insert(name.clone(), Value::Unknown);
            }
        }
        let supported = !steps.iter().any(|step| matches!(step, Step::Unsupported));
        if !supported {
            for value in env.values_mut() {
                *value = Value::Unsupported;
            }
        }
        for step in steps {
            match step {
                Step::Bind(name, expr) => {
                    let value = if supported {
                        self.expr(expr, path, env, depth, generic)
                    } else {
                        Value::Unsupported
                    };
                    env.insert(name.clone(), value);
                }
                Step::Append(name, expr) => {
                    let tail = self.expr(expr, path, env, depth, generic);
                    let base = env.get(name).cloned().unwrap_or(Value::Unknown);
                    env.insert(
                        name.clone(),
                        if supported {
                            concat(base, tail)
                        } else {
                            Value::Unsupported
                        },
                    );
                }
                Step::Effect(expr) => {
                    self.expr(expr, path, env, depth, generic);
                    if !self.known_executor(expr, path) {
                        // Arbitrary effects can mutate a builder passed by
                        // reference. Its previous prefix is no longer proof.
                        for value in env.values_mut() {
                            if matches!(value, Value::Prefix(_, _)) {
                                *value = Value::Unknown;
                            }
                        }
                    }
                }
                Step::Return(expr) => return self.expr(expr, path, env, depth, generic),
                Step::Unsupported => {}
            }
        }
        Value::Unknown
    }

    fn known_executor(&self, expr: &Expr, path: &Path) -> bool {
        match expr {
            Expr::Call { line, spelling, .. } => self.files[path]
                .executors
                .iter()
                .any(|(site_line, name)| site_line == line && name == spelling),
            _ => false,
        }
    }
}
