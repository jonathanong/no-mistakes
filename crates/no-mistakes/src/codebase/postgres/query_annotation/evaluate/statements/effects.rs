use super::{Environment, Evaluator, Step, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn proven_append_only_effect(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: Environment,
        depth: u8,
        generic: bool,
    ) -> bool {
        let Expr::Call { callee, args, .. } = expr else {
            return false;
        };
        if args
            .iter()
            .any(|arg| self.effect_can_mutate(arg, path, env))
        {
            return false;
        }
        let Expr::Name(name) = &**callee else {
            return false;
        };
        let value = self.mapped_parameter_value(env, name).unwrap_or_else(|| {
            self.scopes[env]
                .get(name)
                .cloned()
                .unwrap_or_else(|| self.name(path, name, depth, generic))
        });
        let Value::Function(function, function_path, captured) = value.exposed() else {
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

    pub(in super::super) fn effect_can_mutate(
        &self,
        expr: &Expr,
        path: &Path,
        env: Environment,
    ) -> bool {
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
            Expr::Tagged(tag, _, effects) => {
                !self.tag_trusted(tag, path, env)
                    || effects
                        .iter()
                        .any(|effect| self.effect_can_mutate(effect, path, env))
            }
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
