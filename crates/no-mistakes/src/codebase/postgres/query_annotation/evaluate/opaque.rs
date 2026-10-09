use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn opaque(
        &mut self,
        children: &[Expr],
        targets: &[String],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        self.invalidate_opaque_mapped_targets(targets, *env);
        let values = children
            .iter()
            .map(|child| self.expr(child, path, env, depth, generic))
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        self.opaque_callbacks(&values, depth);
        // Unsupported binding writes must also clear unmodeled bindings such
        // as `arguments`, including canonical aliases captured by callbacks.
        for target in targets {
            self.write_captured_binding(*env, target, &Value::Unknown);
        }
        self.invalidate_opaque_mapped_targets(targets, *env);
        Value::Unknown
    }
}
