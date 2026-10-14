use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn optional_call(
        &mut self,
        call: &Expr,
        children: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let children = if self.files[path].facts.precise_records {
            std::slice::from_ref(call)
        } else {
            children
        };
        self.opaque(children, &[], path, env, context)
    }

    pub(super) fn opaque(
        &mut self,
        children: &[Expr],
        targets: &[String],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let depth = context.0;
        // Read target effects and the RHS before clearing written bindings.
        let values = children
            .iter()
            .map(|child| self.operand(child, path, env, context))
            .collect::<Vec<_>>();
        self.invalidate_opaque_builders(&values);
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
