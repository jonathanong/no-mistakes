use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn projected_primitive_arguments(
        &self,
        args: &[Expr],
        values: &[Value],
        path: &Path,
        env: Environment,
    ) -> bool {
        (args
            .iter()
            .any(|arg| matches!(arg, Expr::Sequence(_) | Expr::Discard(_)))
            || values.iter().any(runtime_primitive))
            && args.iter().zip(values).all(|(arg, value)| {
                (primitive_result(arg) || runtime_primitive(value))
                    && self.handled_deletion_effect(arg, value, path, env)
            })
    }

    pub(super) fn sequence(
        &mut self,
        parts: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let mut result = Value::Unknown;
        let mut handled = true;
        for part in parts {
            let value = self.expr(part, path, env, depth, generic);
            handled &= self.handled_deletion_effect(part, &value, path, *env);
            result = value.exposed();
        }
        Value::Evaluated(Box::new(result), handled)
    }

    pub(super) fn discard(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let value = self.expr(expr, path, env, depth, generic);
        let handled = self.handled_deletion_effect(expr, &value, path, *env);
        Value::Evaluated(Box::new(Value::Unknown), handled)
    }
}

fn runtime_primitive(value: &Value) -> bool {
    match value {
        Value::Primitive | Value::SlotDeletion => true,
        Value::Evaluated(value, _) => runtime_primitive(value),
        _ => false,
    }
}

// Only syntax with a primitive runtime result earns this effect proof;
// an unresolved identifier or arbitrary Unknown value never does.
fn primitive_result(expr: &Expr) -> bool {
    match expr {
        Expr::Primitive | Expr::Text(_) | Expr::Discard(_) => true,
        Expr::Sequence(parts) => parts.last().is_some_and(primitive_result),
        _ => false,
    }
}

#[cfg(test)]
#[path = "result/tests.rs"]
mod tests;
