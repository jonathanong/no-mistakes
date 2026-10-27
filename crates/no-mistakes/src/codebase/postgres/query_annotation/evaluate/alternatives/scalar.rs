use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn scalar_alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Option<Value> {
        // A local scalar read cannot initialize a module, allocate a frame or
        // mutate an alias. Prove every arm before evaluating any of them.
        if !arms.iter().all(|arm| match arm {
            Expr::Unknown
            | Expr::Primitive
            | Expr::Unsupported
            | Expr::Number(_)
            | Expr::Boolean(_)
            | Expr::Text(_) => true,
            Expr::Name(name) => {
                !self.mapped_parameter_unknown(*env, name)
                    && self.scopes[*env].get(name).is_some_and(scalar)
            }
            _ => false,
        }) {
            return None;
        }
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.scalar_alternatives", 1);
        }
        let returned = arms
            .iter()
            .map(|arm| self.expr(arm, path, env, depth, generic))
            .collect();
        Some(super::super::concat::alternative_result(
            returned,
            self.files[path].facts.precise_records,
        ))
    }
}

fn scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Prefix(_, _, None)
            | Value::Number(_)
            | Value::Boolean(_)
            | Value::Unknown
            | Value::Primitive
            | Value::Unsupported
            | Value::Failure(_)
    )
}

#[cfg(test)]
mod tests;
