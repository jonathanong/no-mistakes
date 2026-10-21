use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::codebase::postgres::query_annotation) enum TransientRoot {
    Argument(u64),
    Builder(u64),
    Environment(Environment),
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn with_temporary_roots<T>(&mut self, evaluate: impl FnOnce(&mut Self) -> T) -> T {
        let previous = self.transient_roots.len();
        let value = evaluate(self);
        self.transient_roots.truncate(previous);
        value
    }

    pub(super) fn retain_temporary_roots(&mut self, value: &Value) {
        retain(value, &mut self.transient_roots, &mut FxHashSet::default());
    }

    pub(super) fn retain_temporary_values(&mut self, values: &[Value]) {
        let mut visited = FxHashSet::default();
        for value in values {
            retain(value, &mut self.transient_roots, &mut visited);
        }
    }

    pub(super) fn call(
        &mut self,
        callee: &super::Expr,
        args: &[super::Expr],
        span: (u32, u32),
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        self.with_temporary_roots(|this| {
            this.call_with_roots(callee, args, span, path, env, context)
        })
    }

    pub(super) fn container(
        &mut self,
        expr: &super::Expr,
        children: &[super::Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        self.with_temporary_roots(|this| {
            this.container_with_roots(expr, children, path, env, context)
        })
    }

    pub(super) fn opaque_callbacks(&mut self, arguments: &[Value], depth: u8) {
        self.with_temporary_roots(|this| {
            this.retain_temporary_values(arguments);
            this.opaque_callbacks_with_roots(arguments, depth);
        });
    }
}

fn retain(value: &Value, roots: &mut Vec<TransientRoot>, visited: &mut FxHashSet<*const ()>) {
    match value {
        Value::Array(id) | Value::Arguments(id) => {
            roots.push(TransientRoot::Argument(*id));
        }
        Value::Prefix(_, _, Some(id)) => {
            roots.push(TransientRoot::Builder(*id));
        }
        Value::Function(_, _, env) => {
            roots.push(TransientRoot::Environment(*env));
        }
        Value::Promise(value) | Value::Evaluated(value, _) => retain(value, roots, visited),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values)
            if (values.contains_reference() || values.contains_environment())
                && visited.insert(values.identity()) =>
        {
            for value in values {
                retain(value, roots, visited);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
