use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn atomic(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Option<Value> {
        let (depth, generic) = context;
        Some(match expr {
            Expr::Unknown | Expr::Primitive => Value::Unknown,
            Expr::Number(number) => {
                if self.files[path].facts.precise_records {
                    Value::Number(*number)
                } else {
                    Value::Unknown
                }
            }
            Expr::Boolean(boolean) => {
                if self.files[path].facts.precise_records {
                    Value::Boolean(*boolean)
                } else {
                    Value::Unknown
                }
            }
            Expr::Unsupported => Value::Unsupported,
            Expr::Text(value) => Value::Prefix(value.clone(), true, None),
            Expr::Name(name) => self.mapped_parameter_value(*env, name).unwrap_or_else(|| {
                self.scopes[*env]
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| self.name(path, name, depth, generic))
            }),
            Expr::Function(function) => {
                // Request-owned scope IDs retain live bindings without recursive
                // closure copies or reference cycles between sibling functions.
                Value::Function(Arc::clone(function), path.to_path_buf(), *env)
            }
            _ => return None,
        })
    }
}
