use super::{Environment, Evaluator, Expr, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn failed_binding(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: Environment,
        unsupported: &Value,
        depth: u8,
    ) -> Value {
        let Some(depth) = depth.checked_sub(1) else {
            return self.remaining_callbacks(expr, path, env, unsupported);
        };
        // Preserve inert callable identity, never evaluate an opaque initializer
        // or execute the body merely because its declaration was encountered.
        match expr {
            Expr::Function(function) => {
                let mut function = std::sync::Arc::clone(function);
                std::sync::Arc::make_mut(&mut function).supported = false;
                Value::Function(function, path.to_path_buf(), env)
            }
            Expr::Name(name) => match self.scopes[env].get(name) {
                Some(Value::Function(function, owner, captured)) => {
                    let mut function = std::sync::Arc::clone(function);
                    std::sync::Arc::make_mut(&mut function).supported = false;
                    Value::Function(function, owner.clone(), *captured)
                }
                _ => unsupported.clone(),
            },
            Expr::Record(children, keys) => {
                let values = children
                    .iter()
                    .map(|child| self.failed_binding(child, path, env, unsupported, depth))
                    .collect::<Vec<_>>()
                    .into();
                Value::Record(values, keys.clone())
            }
            Expr::Array(children) => {
                let values = children
                    .iter()
                    .map(|child| self.failed_binding(child, path, env, unsupported, depth))
                    .collect();
                let id = self.next_builder;
                self.next_builder += 1;
                self.argument_objects.insert(id, values);
                Value::Array(id)
            }
            _ => unsupported.clone(),
        }
    }
    fn remaining_callbacks(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: Environment,
        unsupported: &Value,
    ) -> Value {
        // Budget failure keeps only inert identities, not deeper value precision.
        let mut pending = vec![expr];
        let mut callbacks = Vec::new();
        while let Some(expr) = pending.pop() {
            match expr {
                Expr::Record(children, _) | Expr::Array(children) => pending.extend(children),
                Expr::Function(_) | Expr::Name(_) => {
                    callbacks.push(self.failed_binding(expr, path, env, unsupported, 1));
                }
                _ => {}
            }
        }
        Value::References(callbacks.into())
    }
}
